use crate::*;

use syn::{
    self, Attribute, Data, DeriveInput, Field as SynField, Fields as SynFields, Generics, Ident,
    Type, Visibility,
};

use quote::ToTokens;

use proc_macro2::{Span, TokenStream};

mod field_map;
mod type_param_map;

pub(crate) use self::{
    field_map::FieldMap,
    type_param_map::TypeParamMap,

};

//////////////////////////////////////////////////////////////////////////////

/// A type definition(enum,struct,union).
#[derive(Clone, Debug, PartialEq, Hash)]
pub(crate) struct DataStructure<'a> {
    pub(crate) vis: &'a Visibility,
    pub(crate) name: &'a Ident,
    pub(crate) generics: &'a Generics,
    pub(crate) lifetime_count: usize,
    pub(crate) field_count: usize,
    pub(crate) pub_field_count: usize,

    pub(crate) attrs: &'a [Attribute],

    /// Whether this is a struct/union/enum.
    pub(crate) data_variant: DataVariant,

    /// The variants in the type definition.
    ///
    /// If it is a struct or a union this only has 1 element.
    pub(crate) variants: Vec<Struct<'a>>,
}


impl<'a> DataStructure<'a> {
    pub(crate) fn new(
        ast: &'a mut DeriveInput, 
        _arenas: &'a Arenas,
    ) -> Self {
        let name = &ast.ident;

        let data_variant: DataVariant;

        let mut variants = Vec::new();


        match &mut ast.data {
            Data::Enum(enum_) => {
                let override_vis=Some(&ast.vis);

                for (variant,var) in (&mut enum_.variants).into_iter().enumerate() {
                    variants.push(Struct::new(
                        StructParams{
                            discriminant:var.discriminant
                                .as_ref()
                                .map(|(_,v)| v ),
                            variant:variant,
                            attrs:&var.attrs,
                            name:&var.ident,
                            override_vis:override_vis,
                        },
                        &mut var.fields,
                    ));
                }
                data_variant = DataVariant::Enum;
            }
            Data::Struct(struct_) => {
                let override_vis=None;

                variants.push(Struct::new(
                    StructParams{
                        discriminant:None,
                        variant:0,
                        attrs:&[],
                        name:name,
                        override_vis:override_vis,
                    },
                    &mut struct_.fields,
                ));
                data_variant = DataVariant::Struct;
            }

            Data::Union(union_) => {
                let override_vis=None;

                let fields = Some(&union_.fields.named);
                let sk = StructKind::Braced;
                let vari = Struct::with_fields(
                    StructParams{
                        discriminant:None,
                        variant:0,
                        attrs:&[], 
                        name:name, 
                        override_vis:override_vis,
                    },
                    sk, 
                    fields,
                );
                variants.push(vari);
                data_variant = DataVariant::Union;
            }
        }

        let mut field_count=0;
        let mut pub_field_count=0;

        for vari in &variants {
            field_count+=vari.fields.len();
            pub_field_count+=vari.pub_field_count;
        }

        Self {
            vis: &ast.vis,
            name,
            attrs: &ast.attrs,
            generics: &ast.generics,
            lifetime_count:ast.generics.lifetimes().count(),
            data_variant,
            variants,
            field_count,
            pub_field_count,
        }
    }

    pub(crate) fn has_public_fields(&self)->bool{
        self.pub_field_count!=0
    }
}

//////////////////////////////////////////////////////////////////////////////

/// Whether the struct is tupled or not.
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Ord, Eq, Hash)]
pub(crate) enum StructKind {
    /// structs declared using the `struct Name( ... ) syntax.
    Tuple,
    /// structs declared using the `struct Name{ ... }` or `struct name;` syntaxes
    Braced,
}

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Ord, Eq, Hash)]
pub(crate) enum DataVariant {
    Struct,
    Enum,
    Union,
}


#[derive(Copy,Clone, Debug, PartialEq, Hash)]
pub(crate) struct FieldIndex {
    pub(crate) variant:usize,
    pub(crate) pos:usize,
}

//////////////////////////////////////////////////////////////////////////////


#[derive(Copy,Clone)]
struct StructParams<'a>{
    discriminant:Option<&'a syn::Expr>,
    variant:usize,
    attrs: &'a [Attribute],
    name: &'a Ident,
    override_vis:Option<&'a Visibility>,
}

/// A struct/union or a variant of an enum.
#[derive(Clone, Debug, PartialEq, Hash)]
pub(crate) struct Struct<'a> {
    /// The attributes of this `Struct`.
    ///
    /// If this is a struct/union:these is the same as DataStructure.attrs.
    ///
    /// If this is an enum:these are the attributes on the variant.
    pub(crate) attrs: &'a [Attribute],
    /// The name of this `Struct`.
    ///
    /// If this is a struct/union:these is the same as DataStructure.name.
    ///
    /// If this is an enum:this is the name of the variant.
    pub(crate) name: &'a Ident,
    pub(crate) kind: StructKind,
    pub(crate) fields: Vec<Field<'a>>,
    pub(crate) pub_field_count:usize,
    /// The value of this discriminant.
    ///
    /// If this is a Some(_):This is an enum with an explicit discriminant value.
    ///
    /// If this is an None:
    ///     This is either a struct/union or an enum variant without an explicit discriminant.
    pub(crate) discriminant:Option<&'a syn::Expr>,
    _priv: (),
}

impl<'a> Struct<'a> {
    fn new(
        p:StructParams<'a>,
        fields: &'a SynFields,
    ) -> Self {
        let kind = match *fields {
            SynFields::Named { .. } => StructKind::Braced,
            SynFields::Unnamed { .. } => StructKind::Tuple,
            SynFields::Unit { .. } => StructKind::Braced,
        };
        let fields = match fields {
            SynFields::Named(f) => Some(&f.named),
            SynFields::Unnamed(f) => Some(&f.unnamed),
            SynFields::Unit => None,
        };

        Self::with_fields(p, kind, fields)
    }

    fn with_fields<I>(
        p:StructParams<'a>,
        kind: StructKind,
        fields: Option<I>,
    ) -> Self
    where
        I: IntoIterator<Item = &'a SynField>,
    {
        let fields=match fields {
            Some(x) => Field::from_iter(p, x),
            None => Vec::new(),
        };

        let mut pub_field_count=0usize;

        for field in &fields {
            if field.is_public() {
                pub_field_count+=1;
            }
        }

        Self {
            discriminant:p.discriminant,
            attrs:p.attrs,
            name:p.name,
            kind,
            pub_field_count,
            fields,
            _priv: (),
        }
    }

    }

//////////////////////////////////////////////////////////////////////////////

/// Represent a struct field
///
#[derive(Clone, Debug, PartialEq, Hash)]
pub(crate) struct Field<'a> {
    pub(crate) index:FieldIndex,
    pub(crate) attrs: &'a [Attribute],
    pub(crate) vis: &'a Visibility,
    /// identifier for the field,which is either an index(in a tuple struct) or a name.
    pub(crate) ident: FieldIdent<'a>,
    pub(crate) ty: &'a Type,
}

impl<'a> Field<'a> {
    fn new(
        index: FieldIndex,
        field: &'a SynField,
        span: Span,
        override_vis:Option<&'a Visibility>,
    ) -> Self {
        let ident = match field.ident.as_ref() {
            Some(ident) => FieldIdent::Named(ident),
            None => FieldIdent::new_index(index.pos, span),
        };

        Self {
            index,
            attrs: &field.attrs,
            vis: override_vis.unwrap_or(&field.vis),
            ident,
            ty: &field.ty,
        }
    }

    pub(crate) fn is_public(&self)->bool{
        match self.vis {
            Visibility::Public{..}=>true,
            _=>false,
        }
    }

    /// Gets the identifier of this field as an `&Ident`.
    pub(crate) fn ident(&self)->&Ident{
        match &self.ident {
            FieldIdent::Index(_,ident)=>ident,
            FieldIdent::Named(ident)=>ident,
        }
    }

    fn from_iter<I>(
        p:StructParams<'a>,
        fields: I, 
    ) -> Vec<Self>
    where
        I: IntoIterator<Item = &'a SynField>,
    {
        fields
            .into_iter()
            .enumerate()
            .map(|(pos, f)|{ 
                let fi=FieldIndex{variant:p.variant,pos};
                Field::new(fi, f, p.name.span(),p.override_vis)
            })
            .collect()
    }
}

//////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub(crate) enum FieldIdent<'a> {
    Index(usize, Ident),
    Named(&'a Ident),
}

impl<'a> ToTokens for FieldIdent<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match *self {
            FieldIdent::Index(ind, ..) => syn::Index::from(ind).to_tokens(tokens),
            FieldIdent::Named(name) => name.to_tokens(tokens),
        }
    }
}

impl<'a> FieldIdent<'a> {
    fn new_index(index: usize, span: Span) -> Self {
        FieldIdent::Index(index, Ident::new(&format!("field_{}", index), span))
    }
}

