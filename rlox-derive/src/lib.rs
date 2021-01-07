use proc_macro::TokenStream;
use quote::quote;
use syn::{
    self,
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    Expr, ExprRepeat, Path, Token,
};

/// Utility for instantiating arrays of repeating elements that do not implement `Copy`.
///
/// Rust does not allow non-`Copy` values in array repeat expression.
///
/// ```compile_fail
/// struct Foo {}
///
/// let arr = [Foo{}; 256];
/// ```
///
/// This macro generates code for populating an array.
///
/// ```
/// use rlox_derive::array_init;
///
/// struct Foo {}
///
/// // There is a limitation where the type of the element must be passed in.
/// let arr = array_init!(Foo, [Foo{}; 256]);
/// ```
///
/// It accomplishes this by creating an uninitialzed array and populating the elements via a loop.
///
/// See: [MaybeUninit#initializing-an-array-element-by-element](https://doc.rust-lang.org/beta/std/mem/union.MaybeUninit.html#initializing-an-array-element-by-element)
///
/// Related issue: [Constants in array repeat expressions](https://github.com/rust-lang/rust/issues/49147)
///
/// # Safety
///
/// If the given element expression can raise a panic during array initialization, the patially initialized
/// array – and its contents – will not be dropped.
///
/// This is a leak for types that need drop called. Example: `Box<T>`.
#[proc_macro]
pub fn array_init(args: TokenStream) -> TokenStream {
    let arr_construct = parse_macro_input!(args as ArrayConstruct);

    let ArrayConstruct { ty, el, n, .. } = arr_construct;

    let gen = quote! {
        {
            // Implementation taken from `MaybeUninit` documentation.
            use std::mem::{MaybeUninit, self};

            // Create an uninitialized array of `MaybeUninit`. The `assume_init` is
            // safe because the type we are claiming to have initialized here is a
            // bunch of `MaybeUninit`s, which do not require initialization.
            let mut arr: [MaybeUninit<#ty>; #n] = unsafe {
                MaybeUninit::uninit().assume_init()
            };

            // Dropping a `MaybeUninit` does nothing. Thus using raw pointer
            // assignment instead of `ptr::write` does not cause the old
            // uninitialized value to be dropped. Also if there is a panic during
            // this loop, we have a memory leak, but there is no memory safety
            // issue.
            for elem in &mut arr[..] {
                *elem = MaybeUninit::new(#el);
            }

            // `MaybeUninit<T>` is the same size as `T`, with same alignment and everything.
            assert!(mem::size_of::<[MaybeUninit<#ty>; #n]>() == mem::size_of::<[#ty; #n]>());

            // Everything is initialized. Transmute the array to the
            // initialized type.
            unsafe { mem::transmute::<_, [#ty; #n]>(arr) }
        }
    };

    gen.into()
}

/// Fields are boxed otherwise the struct would be very large.
struct ArrayConstruct {
    /// Element type.
    ty: Box<Path>,
    /// Element expression.
    el: Box<Expr>,
    /// Length of array.
    n: Box<Expr>,
    /// Array instantiate expression. `[el; n]`
    #[allow(dead_code)]
    arr: Box<ExprRepeat>,
}

impl Parse for ArrayConstruct {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let punc = Punctuated::<Expr, Token![,]>::parse_terminated(input)?;

        if punc.len() != 2 {
            let msg = &format!(
                "Incorrect number of arguments to macro. Expected 2, got {}.",
                punc.len()
            );
            return Err(syn::Error::new_spanned(punc, msg));
        }

        // Element type
        let expr_path = if let Some(Expr::Path(expr_path)) = punc.first() {
            expr_path
        } else {
            return Err(syn::Error::new_spanned(
                punc.clone(),
                "First argument must be a path or identifier of the element type.",
            ));
        };

        // Array instantiation
        let expr_arr = if let Some(Expr::Repeat(expr_arr)) = punc.iter().skip(1).next() {
            expr_arr
        } else {
            return Err(syn::Error::new_spanned(
                punc.clone(),
                "First argument must be a path or identifier of the element type.",
            ));
        };

        // Element expression.
        let el = &expr_arr.expr;

        // Array size
        let n = &expr_arr.len;

        Ok(Self {
            ty: Box::new(expr_path.path.clone()),
            el: el.clone(),
            n: n.clone(),
            arr: Box::new(expr_arr.clone()),
        })
    }
}
