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
/// use rlox_derive::array;
///
/// struct Foo {}
///
/// // There is a limitation where the type of the element must be passed in.
/// let arr = array!(Foo, [Foo{}; 256]);
/// ```
///
/// It accomplishes this by creating a zeroed array and populating the elements via a loop.
///
/// Related issue: [Constants in array repeat expressions](https://github.com/rust-lang/rust/issues/49147)
#[proc_macro]
pub fn array(args: TokenStream) -> TokenStream {
    let arr_construct = parse_macro_input!(args as ArrayConstruct);

    let ArrayConstruct { ty, el, n, .. } = arr_construct;

    let gen = quote! {
        {
            // SAFETY: Undefined behaviour.
            let mut arr: [#ty; #n] = unsafe { std::mem::zeroed() };

            for i in 0..#n {
                arr[i] = #el;
            }

            arr
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
