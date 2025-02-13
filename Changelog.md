This is the changelog,summarising changes in each version(some minor changes may be ommited).

# 0.7

### 0.7.3

This version constifies many functions,with some mentioned as requiring Rust 1.39 to be const.

These are the constified functions:

```text
abi_stable::std_types::{
    RSlice::from_ref,
    RSlice::from_slice, //from Rust 1.39 onwards

    RSliceMut::{as_ptr, into_mut_ptr},

    RStr::from_str, //from Rust 1.39 onwards
    RStr::{as_rslice, is_empty},

    RString::{as_ptr, as_rstr, from_utf8_unchecked, new},

    RVec::{as_ptr, as_rslice, is_empty, new},
}
```

Now `rstr!()` won't require a literal argument from Rust 1.39 onwards,
it can be any `&str`.



### 0.7.2

Bug fix:

This fixes the version number for abi_stable that is stored in dynamic libraries to 
always be the same as `abi_stable`.
Before this fix the version number was 0.6 which means that you'll have to recompile dynamic libraries of previous patch versions(the 0.7.0 and 0.7.1 versions are yanked because of this).

### 0.7.0

- Added `const fn` constructor functions for 
    `#[sabi_trait]` generated trait objects/DynTrait/RObject.

- Added `RRef<'a,T>` type,as a workaround to allow transmuting `&T` to `&()`.

- Added StableAbi attributes:
    `#[sabi(bounds=""]`:for adding multiple bounds to the StableAbi impl.
    `#[sabi(prefix_bounds=""]`:for adding multiple bounds to the PrefixTypeTrait impl.
    `#[sabi(sabi_opaque_fields]`:
        To treat a field as opaque while still requiring it to impl StableAbi.
    `#[sabi(sabi_opaque_fields]`:
        To treat every field as opaque while still requiring them to impl StableAbi.
    `#[sabi(shared_stableabi())]`:
        replaces the default `T:StableAbi` bound with `T:SharedStableAbi`
    `#[sabi(phantom_const_param="<expr>")]` :
        This adds `<expr>` as a virtual const parameter,
        that is checked for equality like every other const parameter.

- Added ConstGeneric,to have proper const-generics support,
    this allows any type that implements `Eq+Debug+StableAbi+'static` to be 
    used as a const-parameter.

- Added macros for constructing RVec/Tuple0-4/RStr/RSlice/NulStr

- Added NulStr,a nul terminated utf8 string slice.

- Rewrote how type layout constants are represented to be significantly smaller.
    Most of the optimizations are described in the 
    `ffdd68fef8d445d7d91972b0d751db80df887ec4` commit
    (there were some tweaks after that commit,but it's mostly correct).

- Now `#[sabi_trait]trait Trait:'static{}` allows the trait object
    to be constructed from a non-'static reference to a `'static` type,
    removing lifetime supertraits from `Trait` in the generated code.
    Now generates a `Trait_Bound` trait with all the supertraits(including lifetimes).

- Renamed `#[sabi(field_bound="")]` to `#[sabi(accessor_bound="")]`,
    because it only adds a bound to the accessor methods of prefix types.

- Merged the `abi_stable_derive_lib` crate into the `abi_stable_derive` crate.

- Changed error reporting in proc macros to report as many errors as possible,
    pointing at what the cause of the error is.

- Added reborrowing support to `#[sabi_trait]` generated trait objects.

- Changed TypeInfo to use `std::any::type_name` 
    to print the type in error messages from 1.38 onwards.

- Renamed DynTrait/RObject unerasure methods for the common case.

- Split TransmuteElement into
    the CanTransmuteElement marker trait and 
    the TransmuteElement extension trait

- Now forbidding type macros,they will be allowed once 
    referenced lifetime can be detected inside macro invocations.

- Added Debug and Display support in RObject.

- Added a way to add extra checks to type layouts at load time with 
    `#[sabi(extra_checks="")]`,passing a type that implements ExtraChecks.
    Replaced uses of `#[sabi(tag="...")]` by DynTrait/RObject/NonExhaustive.

- Made it possible to borrow from self in SerializeProxyType.

# 0.6

### 0.6.3

- Added documentation examples to virtually every type/method in 
    `abi_stable::{external_types,sabi_types,std_types}`

- Added a few methods/associated functions because examples made it 
    obvious that they were necessary.

- Changed RBoxError_ downcast methods to downcast through a 
    `Box<dyn Error+ ... >` if it wraps one.
    
    This involves a tiny breaking change where downcast now requires 
    `std::error::Error` to be implemented by the error being downcasted.
    This breaking change should not be a problem,
    since `RBoxError::{new,from_box,from}` requires that the type implements the `Error` trait,
    meaning that one can only sensibly downcast to types that implement the trait

- Added `ROnce::NEW` associated constant as a workaround for a compiler bug

- Added `abi_stable::inline_storage::alignment::AlignToUsize`

### 0.6.2

- Added the `#[derive(GetStaticEquivalent)]` derive macro.

- Added `#[sabi(impl_InterfaceType())]` helper attribute to `#[derive(StableAbi)]` 
    and `#[derive(GetStaticEquivalent)]`.

- Replaced most uses of `impl_InterfaceType!{}` with the helper attribute.

- Added comments explaining abi_stable concepts in examples.

### 0.6.0

- Implemented nonexhastive enum derivation and NonExhaustive wrapper type,
    with documentation on how to use them,
    and an extra set of example crates in "examples/2_nonexhaustive/\*".

- Rewrote a lot of the code generated by #[sabi_trait]:
    
    - It now generates a `struct Trait_TO` instead of a type alias,
        wrapping the underlying implementation (DynTrait or RObject).
    
    - Transformed the constructors into associated functions of `Trait_TO`,
        adding the `Trait_TO::from_sabi` to wrap the underlying implementation.
    
    - Added impls delegating the supertraits to the underlying implementation.
    
    - Automatically add supertraits of supertraits.
    
    - Fixed support for Iterator/DoubleEndedIterator,
        parsing the supertrait bound to detect the Iterator Item type.
    
    - Replaced Trait_Marker with `Trait_Interface<'a,'b,A,B,AssocTypeA,AssocTypeB >`.

    - Added `erasability:Erasability` parameter to constructor functions,
        to pass `TU_Unerasable` and `TU_Opaque` by value(instead of as a type parameter).


- Added #[StableAbi] attributes:

    - `#[sabi(phantom_field="name:type")]`

    - `#[sabi(phantom_type_param="type")]`

    - `#[sabi(not_stableabi())]`:to accept type parameters that only implement GetStaticEquivalent_

    - `#[sabi(unsafe_change_type="SomeType")]`:to change the type of the field in the type layout constant.


- Added `#[unsafe_no_layout_constant]` attribute to `#[export_root_module]`,
    to have a abi_stable dynamic library without storing the type layout of the root module.

- Changed (de)serialization of DynTrait/nonexhaustive enums to use proxy types,
    an intermediate type that the type is converted from/to in between (de)serializing.

- Removed where clause in DynTrait type definition.
    Removed `'borr` and IteratorItem associated type from InterfaceBound.
    Changed IteratorItemOrDefault to get the Iterator Item type of a particular InterfaceType.

- Allow reborrowed DynTrait to be unerasable in more situations.

- Added std::error::Error support for DynTrait and #[sabi_trait] traits.

- Added ffi-safe equivalent of serde_json::value::RawValue,
    mostly for use as a (de)serialization proxy type.

- Added GetStaticEquivalent_ to get the `'static` equivalent of a type for type checking.

- Fixed runtime type checking soundness bugs:
    
    - where using `#[sabi(unconstrained())]` would cause the type parameter to be ignored 
        when computing the UTypeId for the type.
        Renamed the attribute to #[sabi(unsafe_unconstrained())].

    - where non-StableAbi fields were treated as opaque,
        even though the `#[sabi(unsafe_opaque_field)]` attribute wasn't applied to them.

    - where checking prefix types against the global view
        didn't return an error when nested fields had errors.

- Made LibHeader safely usable by reference,required making changes to make it thread safe
    (it didn't need to be before).

- Removed bounds from unconditional accessors in prefix_structs.

- Improved how type layout errors are `Display`ed,making them significantly more specific,
    where previously it printed the entire type layout(it could overflow the terminal buffer).

- Moved abi_stability::type_layout to root module,with abi_stability::tagging inside.

- Replaced the True/False in InterfaceType associated types with 
    `Implemented<Trait>` and `Unimplemented<Trait>`,to improve compile-time error messages.

- Added `#[sabi_extern_fn]` attribute,to replace many uses of 
    `extern fn foo(){ extern_fn_panic_handling!{} }`

- Removed suffix from RCmpOrdering variants.

- Moved InlineStorage from sabi_types::rsmallbox to its own top-level module.

- Added Spans (the region of code tokens come from) to most generated code,
    to improve error messages from within macro generated code.



# 0.5

- Added MovePtr type and OwnedPointer trait,mostly for `#[sabi_trait]`.

- Implemented `#[sabi_trait]` attribute for generating ffi-safe trait objects 
    from a trait definition.

- Implemented RObject,the default backend type of `#[sabi_trait]`.

- Made generated type layout constants significantly smaller(in the binary),
    by changing representation of fields and functions to structs of arrays(approximately).

- Added unchecked versions of library loading methods in LibHeader.

- Moved example crates to their own numbered subfolders.

- Added `#[sabi_trait]` example crate,which implement a basic plugin system.

- Moved some top-level abi_stable modules 
    (ignored_wrapper,late_static_ref,return_value_equality,version) 
    to sabi_types.


# 0.4

- Added basic module reflection,changing a few details of how layout is represented.

- Created the sabi_extract tool that converts the module structure of an 
    abi_stable dynamic library to json.

- Streamlined how modules are exported,
    removing the need to specify the LOADER_FN in the RootModule trait,
    as well as constructing the module using the `RootModule::load_*` functions.

- Changed how the root module is accessed after it's loaded,
    using the `<RootMod as RootModule>::get_module` function.

- Added fn root_module_statics to declare the statics associated with a RootModule,
    as well as the `declare_root_module_statics` macro to implement it.

- Changed `RootModule::raw_library_ref` to `RootModule::get_raw_library` ,
    returning the already loaded RawLibrary instead of allowing the user 
    to initialize it themselves.

- Changed how all libraries are loaded so that the abi_stable version they 
    use can be checked,mentioning the abi_stable version in the returned error.

- Renamed `Library` to `RawLibrary`.

- Now the RawLibrary is unloaded after layout checking fails,
    leaking it if layout checking passes instead of doing so when it's loaded.

- Added `#[sabi(refl(pub_getter=" function_name "))]` 
    attribute for code generation (using the layout constant for a type),
    to determine how to access private fields(otherwise they're inaccesible).

- Renamed  `export_sabi_module` to `export_root_module`.

- Added RMutex/RRwLock/ROnce,wrapping parking_lot types.

- Added ffi-safe wrappers for crossbeam channels.

- Added support for #[repr(<IntegerType>)],for enums.

- Added checking of enum discriminants(supports all integer types up to u64/i64).

- Renamed LazyStaticRef to LateStaticRef,made if ffi-safe.





# 0.3 

- Dropped support for 1.33 (no requiring 1.34) due to 
    an ICE caused by associated types in associated constants.

- Renamed VirtualWrapper to DynTrait,moving `I:InterfaceType` to second type parameter.

- Added tags,a dynamically typed data structure used 
    when checking the layout of types at runtime.

- DynTrait can now be constructed from non-`'static` types,
    using `DynTrait::from_borrowìng_*`.

- Added conditional accessors to prefix-types,
    allowing those fields to have any type if disabled 
    (so long as they don't change in size/alignment)

- Added these conditional traits to DynTrait:
    - Send
    - Sync.
    - Iterator
    - DoubleEndedIterator
    - std::fmt::Write
    - std::io::{Write,Seek,Read,BufRead}

- Improved documentation of DynTrait,including multiple examples,
    and how to make a pointer compatible with it.

- Improved the std_types error types,
    to be almost the same as the ones in the standard library.

- Added reborrowing to DynTrait,
    going from DynTrait<'a,P<()>,I> to DynTrait<'a,&(),I>/DynTrait<'a,&mut (),I> .

- Added impl_InterfaceType macro to implement InterfaceType,
    emulating default associated types.

- Changed RCow to be closer to its standard library equivalent.

- Added methods/documentation to ROption/RResult.

- Added RHashMap,with an API very close to the standard HashMap.

# 0.2

- Added SharedStableAbi trait to implement prefix-types (vtables and modules).

- Added a "StaticEquivalent:'static" associated type to StableAbi/SharedStableAbi 
    to construct a type-id from any type,for checking their layout only once

- Added impl_InterfaceType macro for 
    implementing InterfaceType with default associated types.

- Tightened safety around phantom type parameters,
    requiring every type to appear in the layout constant of the type.

- Implemented prefix-types,for extensible vtables/modules,
    along with rewriting existing VTables and modules to use them.

- Implemented private multi-key map for layout checking
    (it's not public purely for documentation reasons).

- Moved example crates to example folder

- Replaced LibraryTrait/ModuleTrait with RootModule trait,
    only allowing the root module to be exported.

- Improved example 0,adding a readme,
    and an example of serializing/deserializing json commands.

- Added documentation for interactions the library has with unsafe code/
    how to write unsafe code that uses the library.

- Many small changes to documentation.
