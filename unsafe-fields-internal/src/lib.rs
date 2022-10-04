use proc_macro::*;

#[proc_macro_attribute]
pub fn unsafe_fields(_inner: TokenStream, item: TokenStream) -> TokenStream {
    let mut new = Vec::<TokenTree>::new();
    let mut item = item.into_iter();
    let mut inner = loop {
        if let Some(next) = item.next() {
            match next {
                TokenTree::Group(g) if g.delimiter() == Delimiter::Brace => {
                    break g;
                }
                TokenTree::Ident(ident) if matches!(&*ident.to_string(), "enum" | "union") => {
                    panic!("expected struct");
                }
                rest => new.push(rest),
            }
        } else {
            panic!("Could not find field block.");
        }
    }
    .stream()
    .into_iter();
    let mut fields = vec![];
    while let Some(next) = parse_field(&mut inner) {
        fields.push(next);
    }
    new.push(TokenTree::Group(Group::new(
        Delimiter::Brace,
        fields.into_iter().flat_map(Field::into_tokens).collect(),
    )));
    TokenStream::from_iter(new)
}

fn parse_field(s: &mut token_stream::IntoIter) -> Option<Field> {
    let mut attrs = vec![];
    let mut next_token;
    let mut unsafety = false;
    // gobble attributes
    loop {
        if let Some(next) = s.next() {
            match next {
                TokenTree::Punct(punct) if punct.as_char() == '#' => {
                    let next = s.next().expect("expected attribute body.");
                    match next {
                        TokenTree::Group(group) if group.delimiter() == Delimiter::Bracket => {
                            if let Some(unsafe_field) = group
                                .stream()
                                .into_iter()
                                .map(Some)
                                .reduce(|_, _| None)
                                .and_then(|s| s)
                            {
                                if unsafe_field.to_string() == "unsafe_field" {
                                    if unsafety {
                                        panic!("unsafe_field specified twice.");
                                    }
                                    unsafety = true;
                                    continue;
                                }
                            }
                            attrs.push(TokenTree::Punct(punct));
                            attrs.push(TokenTree::Group(group));
                        }
                        r => panic!("expected attribute body, found '{r}'"),
                    }
                }
                TokenTree::Ident(ident) => {
                    next_token = Some(TokenTree::Ident(ident));
                    break;
                }
                r => panic!("expected `pub`, identifier or attribute, found '{r}'",),
            }
        } else {
            if attrs.is_empty() {
                return None;
            } else {
                panic!("expected more tokens.");
            }
        }
    }
    let mut vis = Visibility::Inherited;
    // get visibility
    while let Some(next) = next_token.take().or_else(|| s.next()) {
        match next {
            TokenTree::Ident(ident) => match &*ident.to_string() {
                "pub" => {
                    let pub_token = Pub { ident };
                    if matches!(vis, Visibility::Inherited) {
                        let next = s
                            .next()
                            .expect("expected visibilty restriction or identifier.");
                        match next {
                            TokenTree::Group(group)
                                if group.delimiter() == Delimiter::Parenthesis =>
                            {
                                vis = Visibility::Restricted(VisRestricted {
                                    pub_token,
                                    restrict: group,
                                });
                            }
                            r => {
                                vis = Visibility::Public(pub_token);
                                next_token = Some(r);
                            }
                        }
                    } else {
                        panic!("Visibility specified twice");
                    }
                    break;
                }
                _name => {
                    next_token = Some(TokenTree::Ident(ident));
                    break;
                }
            },
            r => panic!("expected `pub` or identifier, found '{r}'"),
        }
    }
    let ident = match next_token
        .take()
        .or_else(|| s.next())
        .expect("expected identifier.")
    {
        TokenTree::Ident(ident) => ident,
        r => panic!("expected identifier, found '{r}'."),
    };
    let colon = match s.next().expect("expected `:`.") {
        TokenTree::Punct(punct) if punct.as_char() == ':' => punct,
        r => panic!("expected `:`, found '{r}'."),
    };
    let mut ty = vec![];
    // number of `<` we are in:
    let mut nesting = 0;
    while let Some(next) = s.next() {
        match next {
            TokenTree::Punct(punct) if nesting == 0 && punct.as_char() == ',' => break,
            TokenTree::Punct(punct) if punct.as_char() == '<' => {
                nesting += 1;
                ty.push(TokenTree::Punct(punct));
            }
            TokenTree::Punct(punct) if punct.as_char() == '>' => {
                assert!(nesting > 0);
                nesting -= 1;
                ty.push(TokenTree::Punct(punct));
            }
            rest => ty.push(rest),
        }
    }
    Some(Field {
        attrs,
        vis,
        unsafety,
        ident,
        colon,
        ty,
    })
}

// code taken from `syn`, changes noted in comments
mod syn {
    use proc_macro::*;

    #[derive(Debug)]
    pub struct Field {
        pub attrs: Vec<TokenTree>, // just tokens
        pub vis: Visibility,
        pub unsafety: bool,     // this is new
        pub ident: Ident,       // not an option
        pub colon: Punct,       // not an option, just a punct
        pub ty: Vec<TokenTree>, // also just tokens
    }

    impl Field {
        pub fn into_tokens(self) -> Vec<TokenTree> {
            let mut vec = self.attrs;
            self.vis.to_tokens(&mut vec);
            vec.push(TokenTree::Ident(self.ident));
            vec.push(TokenTree::Punct(self.colon));
            if self.unsafety {
                vec.push(TokenTree::Punct(Punct::new(':', Spacing::Joint)));
                vec.push(TokenTree::Punct(Punct::new(':', Spacing::Alone)));
                vec.push(TokenTree::Ident(Ident::new(
                    "unsafe_fields",
                    Span::call_site(),
                )));
                vec.push(TokenTree::Punct(Punct::new(':', Spacing::Joint)));
                vec.push(TokenTree::Punct(Punct::new(':', Spacing::Alone)));
                vec.push(TokenTree::Ident(Ident::new(
                    "UnsafeField",
                    Span::call_site(),
                )));
                vec.push(TokenTree::Punct(Punct::new('<', Spacing::Alone)));
            }
            vec.extend(self.ty);
            if self.unsafety {
                vec.push(TokenTree::Punct(Punct::new('>', Spacing::Alone)));
            }
            vec.push(TokenTree::Punct(Punct::new(',', Spacing::Alone)));
            vec
        }
    }

    #[derive(Debug)]
    pub enum Visibility {
        Public(Pub),
        // not needed:
        // Crate(VisCrate),
        Restricted(VisRestricted),
        Inherited,
    }

    impl Visibility {
        pub fn to_tokens(self, vec: &mut Vec<TokenTree>) {
            match self {
                Visibility::Public(p) => {
                    vec.push(TokenTree::Ident(p.ident));
                }
                Visibility::Restricted(r) => {
                    vec.push(TokenTree::Ident(r.pub_token.ident));
                    vec.push(TokenTree::Group(r.restrict));
                }
                Visibility::Inherited => {}
            }
        }
    }

    #[derive(Debug)]
    pub struct VisRestricted {
        pub pub_token: Pub,
        pub restrict: Group, // just a group
    }

    #[derive(Debug)]
    pub struct Pub {
        pub ident: Ident, // just an ident
    }
}
use syn::*;
