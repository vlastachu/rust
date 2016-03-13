// Copyright 2012-2014 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! A Folder represents an AST->AST fold; it accepts an AST piece,
//! and returns a piece of the same type. So, for instance, macro
//! expansion is a Folder that walks over an AST and produces another
//! AST.
//!
//! Note: using a Folder (other than the MacroExpander Folder) on
//! an AST before macro expansion is probably a bad idea. For instance,
//! a folder renaming item names in a module will miss all of those
//! that are created by the expansion of a macro.

use ast::*;
use ast;
use attr::{ThinAttributes, ThinAttributesExt};
use ast_util;
use codemap::{respan, Span, Spanned};
use parse::token;
use ptr::P;
use util::small_vector::SmallVector;
use util::move_map::MoveMap;

use std::rc::Rc;

pub trait Folder : Sized {
    // Any additions to this trait should happen in form
    // of a call to a public `noop_*` function that only calls
    // out to the folder again, not other `noop_*` functions.
    //
    // This is a necessary API workaround to the problem of not
    // being able to call out to the super default method
    // in an overridden default method.

    fn fold_crate(&mut self, c: Crate) -> Crate {
        noop_fold_crate(c, self)
    }

    fn fold_meta_items(&mut self, meta_items: Vec<P<MetaItem>>) -> Vec<P<MetaItem>> {
        noop_fold_meta_items(meta_items, self)
    }

    fn fold_meta_item(&mut self, meta_item: P<MetaItem>) -> P<MetaItem> {
        noop_fold_meta_item(meta_item, self)
    }

    fn fold_view_path(&mut self, view_path: P<ViewPath>) -> P<ViewPath> {
        noop_fold_view_path(view_path, self)
    }

    fn fold_foreign_item(&mut self, ni: ForeignItem) -> ForeignItem {
        noop_fold_foreign_item(ni, self)
    }

    fn fold_item(&mut self, i: P<Item>) -> SmallVector<P<Item>> {
        noop_fold_item(i, self)
    }

    fn fold_item_simple(&mut self, i: Item) -> Item {
        noop_fold_item_simple(i, self)
    }

    fn fold_struct_field(&mut self, sf: StructField) -> StructField {
        noop_fold_struct_field(sf, self)
    }

    fn fold_item_kind(&mut self, i: ItemKind) -> ItemKind {
        noop_fold_item_kind(i, self)
    }

    fn fold_trait_item(&mut self, i: TraitItem) -> SmallVector<TraitItem> {
        noop_fold_trait_item(i, self)
    }

    fn fold_impl_item(&mut self, i: ImplItem) -> SmallVector<ImplItem> {
        noop_fold_impl_item(i, self)
    }

    fn fold_fn_decl(&mut self, d: P<FnDecl>) -> P<FnDecl> {
        noop_fold_fn_decl(d, self)
    }

    fn fold_block(&mut self, b: P<Block>) -> P<Block> {
        noop_fold_block(b, self)
    }

    fn fold_stmt(&mut self, s: Stmt) -> SmallVector<Stmt> {
        noop_fold_stmt(s, self)
    }

    fn fold_arm(&mut self, a: Arm) -> Arm {
        noop_fold_arm(a, self)
    }

    fn fold_pat(&mut self, p: P<Pat>) -> P<Pat> {
        noop_fold_pat(p, self)
    }

    fn fold_decl(&mut self, d: P<Decl>) -> SmallVector<P<Decl>> {
        noop_fold_decl(d, self)
    }

    fn fold_expr(&mut self, e: P<Expr>) -> P<Expr> {
        e.map(|e| noop_fold_expr(e, self))
    }

    fn fold_opt_expr(&mut self, e: P<Expr>) -> Option<P<Expr>> {
        noop_fold_opt_expr(e, self)
    }

    fn fold_exprs(&mut self, es: Vec<P<Expr>>) -> Vec<P<Expr>> {
        noop_fold_exprs(es, self)
    }

    fn fold_ty(&mut self, t: P<Ty>) -> P<Ty> {
        noop_fold_ty(t, self)
    }

    fn fold_ty_binding(&mut self, t: TypeBinding) -> TypeBinding {
        noop_fold_ty_binding(t, self)
    }

    fn fold_mod(&mut self, m: Mod) -> Mod {
        noop_fold_mod(m, self)
    }

    fn fold_foreign_mod(&mut self, nm: ForeignMod) -> ForeignMod {
        noop_fold_foreign_mod(nm, self)
    }

    fn fold_variant(&mut self, v: Variant) -> Variant {
        noop_fold_variant(v, self)
    }

    fn fold_ident(&mut self, i: Ident) -> Ident {
        noop_fold_ident(i, self)
    }

    fn fold_usize(&mut self, i: usize) -> usize {
        noop_fold_usize(i, self)
    }

    fn fold_path(&mut self, p: Path) -> Path {
        noop_fold_path(p, self)
    }

    fn fold_path_parameters(&mut self, p: PathParameters) -> PathParameters {
        noop_fold_path_parameters(p, self)
    }

    fn fold_angle_bracketed_parameter_data(&mut self, p: AngleBracketedParameterData)
                                           -> AngleBracketedParameterData
    {
        noop_fold_angle_bracketed_parameter_data(p, self)
    }

    fn fold_parenthesized_parameter_data(&mut self, p: ParenthesizedParameterData)
                                         -> ParenthesizedParameterData
    {
        noop_fold_parenthesized_parameter_data(p, self)
    }

    fn fold_local(&mut self, l: P<Local>) -> P<Local> {
        noop_fold_local(l, self)
    }

    fn fold_mac(&mut self, _mac: Mac) -> Mac {
        panic!("fold_mac disabled by default");
        // NB: see note about macros above.
        // if you really want a folder that
        // works on macros, use this
        // definition in your trait impl:
        // fold::noop_fold_mac(_mac, self)
    }

    fn fold_explicit_self(&mut self, es: ExplicitSelf) -> ExplicitSelf {
        noop_fold_explicit_self(es, self)
    }

    fn fold_explicit_self_kind(&mut self, es: SelfKind) -> SelfKind {
        noop_fold_explicit_self_kind(es, self)
    }

    fn fold_lifetime(&mut self, l: Lifetime) -> Lifetime {
        noop_fold_lifetime(l, self)
    }

    fn fold_lifetime_def(&mut self, l: LifetimeDef) -> LifetimeDef {
        noop_fold_lifetime_def(l, self)
    }

    fn fold_attribute(&mut self, at: Attribute) -> Option<Attribute> {
        noop_fold_attribute(at, self)
    }

    fn fold_arg(&mut self, a: Arg) -> Arg {
        noop_fold_arg(a, self)
    }

    fn fold_generics(&mut self, generics: Generics) -> Generics {
        noop_fold_generics(generics, self)
    }

    fn fold_trait_ref(&mut self, p: TraitRef) -> TraitRef {
        noop_fold_trait_ref(p, self)
    }

    fn fold_poly_trait_ref(&mut self, p: PolyTraitRef) -> PolyTraitRef {
        noop_fold_poly_trait_ref(p, self)
    }

    fn fold_variant_data(&mut self, vdata: VariantData) -> VariantData {
        noop_fold_variant_data(vdata, self)
    }

    fn fold_lifetimes(&mut self, lts: Vec<Lifetime>) -> Vec<Lifetime> {
        noop_fold_lifetimes(lts, self)
    }

    fn fold_lifetime_defs(&mut self, lts: Vec<LifetimeDef>) -> Vec<LifetimeDef> {
        noop_fold_lifetime_defs(lts, self)
    }

    fn fold_ty_param(&mut self, tp: TyParam) -> TyParam {
        noop_fold_ty_param(tp, self)
    }

    fn fold_ty_params(&mut self, tps: P<[TyParam]>) -> P<[TyParam]> {
        noop_fold_ty_params(tps, self)
    }

    fn fold_tt(&mut self, tt: &TokenTree) -> TokenTree {
        noop_fold_tt(tt, self)
    }

    fn fold_tts(&mut self, tts: &[TokenTree]) -> Vec<TokenTree> {
        noop_fold_tts(tts, self)
    }

    fn fold_token(&mut self, t: token::Token) -> token::Token {
        noop_fold_token(t, self)
    }

    fn fold_interpolated(&mut self, nt: token::Nonterminal) -> token::Nonterminal {
        noop_fold_interpolated(nt, self)
    }

    fn fold_opt_lifetime(&mut self, o_lt: Option<Lifetime>) -> Option<Lifetime> {
        noop_fold_opt_lifetime(o_lt, self)
    }

    fn fold_opt_bounds(&mut self, b: Option<TyParamBounds>)
                       -> Option<TyParamBounds> {
        noop_fold_opt_bounds(b, self)
    }

    fn fold_bounds(&mut self, b: TyParamBounds)
                       -> TyParamBounds {
        noop_fold_bounds(b, self)
    }

    fn fold_ty_param_bound(&mut self, tpb: TyParamBound) -> TyParamBound {
        noop_fold_ty_param_bound(tpb, self)
    }

    fn fold_mt(&mut self, mt: MutTy) -> MutTy {
        noop_fold_mt(mt, self)
    }

    fn fold_field(&mut self, field: Field) -> Field {
        noop_fold_field(field, self)
    }

    fn fold_where_clause(&mut self, where_clause: WhereClause)
                         -> WhereClause {
        noop_fold_where_clause(where_clause, self)
    }

    fn fold_where_predicate(&mut self, where_predicate: WherePredicate)
                            -> WherePredicate {
        noop_fold_where_predicate(where_predicate, self)
    }

    fn new_id(&mut self, i: NodeId) -> NodeId {
        i
    }

    fn new_span(&mut self, sp: Span) -> Span {
        sp
    }
}

pub fn noop_fold_meta_items<T: Folder>(meta_items: Vec<P<MetaItem>>, fld: &mut T)
                                       -> Vec<P<MetaItem>> {
    meta_items.move_map(|x| fld.fold_meta_item(x))
}

pub fn noop_fold_view_path<T: Folder>(view_path: P<ViewPath>, fld: &mut T) -> P<ViewPath> {
    view_path.map(|Spanned {node, span}| Spanned {
        node: match node {
            ViewPathSimple(ident, path) => {
                ViewPathSimple(ident, fld.fold_path(path))
            }
            ViewPathGlob(path) => {
                ViewPathGlob(fld.fold_path(path))
            }
            ViewPathList(path, path_list_idents) => {
                ViewPathList(fld.fold_path(path),
                             path_list_idents.move_map(|path_list_ident| {
                                Spanned {
                                    node: match path_list_ident.node {
                                        PathListItemKind::Ident { id, name, rename } =>
                                            PathListItemKind::Ident {
                                                id: fld.new_id(id),
                                                rename: rename,
                                                name: name
                                            },
                                        PathListItemKind::Mod { id, rename } =>
                                            PathListItemKind::Mod {
                                                id: fld.new_id(id),
                                                rename: rename
                                            }
                                    },
                                    span: fld.new_span(path_list_ident.span)
                                }
                             }))
            }
        },
        span: fld.new_span(span)
    })
}

pub fn fold_attrs<T: Folder>(attrs: Vec<Attribute>, fld: &mut T) -> Vec<Attribute> {
    attrs.move_flat_map(|x| fld.fold_attribute(x))
}

pub fn fold_thin_attrs<T: Folder>(attrs: ThinAttributes, fld: &mut T) -> ThinAttributes {
    attrs.map_thin_attrs(|v| fold_attrs(v, fld))
}

pub fn noop_fold_arm<T: Folder>(Arm {attrs, pats, guard, body}: Arm, fld: &mut T) -> Arm {
    Arm {
        attrs: fold_attrs(attrs, fld),
        pats: pats.move_map(|x| fld.fold_pat(x)),
        guard: guard.map(|x| fld.fold_expr(x)),
        body: fld.fold_expr(body),
    }
}

pub fn noop_fold_decl<T: Folder>(d: P<Decl>, fld: &mut T) -> SmallVector<P<Decl>> {
    d.and_then(|Spanned {node, span}| match node {
        DeclKind::Local(l) => SmallVector::one(P(Spanned {
            node: DeclKind::Local(fld.fold_local(l)),
            span: fld.new_span(span)
        })),
        DeclKind::Item(it) => fld.fold_item(it).into_iter().map(|i| P(Spanned {
            node: DeclKind::Item(i),
            span: fld.new_span(span)
        })).collect()
    })
}

pub fn noop_fold_ty_binding<T: Folder>(b: TypeBinding, fld: &mut T) -> TypeBinding {
    TypeBinding {
        id: fld.new_id(b.id),
        ident: b.ident,
        ty: fld.fold_ty(b.ty),
        span: fld.new_span(b.span),
    }
}

pub fn noop_fold_ty<T: Folder>(t: P<Ty>, fld: &mut T) -> P<Ty> {
    t.map(|Ty {id, node, span}| Ty {
        id: fld.new_id(id),
        node: match node {
            TyKind::Infer => node,
            TyKind::Vec(ty) => TyKind::Vec(fld.fold_ty(ty)),
            TyKind::Ptr(mt) => TyKind::Ptr(fld.fold_mt(mt)),
            TyKind::Rptr(region, mt) => {
                TyKind::Rptr(fld.fold_opt_lifetime(region), fld.fold_mt(mt))
            }
            TyKind::BareFn(f) => {
                TyKind::BareFn(f.map(|BareFnTy {lifetimes, unsafety, abi, decl}| BareFnTy {
                    lifetimes: fld.fold_lifetime_defs(lifetimes),
                    unsafety: unsafety,
                    abi: abi,
                    decl: fld.fold_fn_decl(decl)
                }))
            }
            TyKind::Tup(tys) => TyKind::Tup(tys.move_map(|ty| fld.fold_ty(ty))),
            TyKind::Paren(ty) => TyKind::Paren(fld.fold_ty(ty)),
            TyKind::Path(qself, path) => {
                let qself = qself.map(|QSelf { ty, position }| {
                    QSelf {
                        ty: fld.fold_ty(ty),
                        position: position
                    }
                });
                TyKind::Path(qself, fld.fold_path(path))
            }
            TyKind::ObjectSum(ty, bounds) => {
                TyKind::ObjectSum(fld.fold_ty(ty),
                            fld.fold_bounds(bounds))
            }
            TyKind::FixedLengthVec(ty, e) => {
                TyKind::FixedLengthVec(fld.fold_ty(ty), fld.fold_expr(e))
            }
            TyKind::Typeof(expr) => {
                TyKind::Typeof(fld.fold_expr(expr))
            }
            TyKind::PolyTraitRef(bounds) => {
                TyKind::PolyTraitRef(bounds.move_map(|b| fld.fold_ty_param_bound(b)))
            }
            TyKind::Mac(mac) => {
                TyKind::Mac(fld.fold_mac(mac))
            }
        },
        span: fld.new_span(span)
    })
}

pub fn noop_fold_foreign_mod<T: Folder>(ForeignMod {abi, items}: ForeignMod,
                                        fld: &mut T) -> ForeignMod {
    ForeignMod {
        abi: abi,
        items: items.move_map(|x| fld.fold_foreign_item(x)),
    }
}

pub fn noop_fold_variant<T: Folder>(v: Variant, fld: &mut T) -> Variant {
    Spanned {
        node: Variant_ {
            name: v.node.name,
            attrs: fold_attrs(v.node.attrs, fld),
            data: fld.fold_variant_data(v.node.data),
            disr_expr: v.node.disr_expr.map(|e| fld.fold_expr(e)),
        },
        span: fld.new_span(v.span),
    }
}

pub fn noop_fold_ident<T: Folder>(i: Ident, _: &mut T) -> Ident {
    i
}

pub fn noop_fold_usize<T: Folder>(i: usize, _: &mut T) -> usize {
    i
}

pub fn noop_fold_path<T: Folder>(Path {global, segments, span}: Path, fld: &mut T) -> Path {
    Path {
        global: global,
        segments: segments.move_map(|PathSegment {identifier, parameters}| PathSegment {
            identifier: fld.fold_ident(identifier),
            parameters: fld.fold_path_parameters(parameters),
        }),
        span: fld.new_span(span)
    }
}

pub fn noop_fold_path_parameters<T: Folder>(path_parameters: PathParameters, fld: &mut T)
                                            -> PathParameters
{
    match path_parameters {
        PathParameters::AngleBracketed(data) =>
            PathParameters::AngleBracketed(fld.fold_angle_bracketed_parameter_data(data)),
        PathParameters::Parenthesized(data) =>
            PathParameters::Parenthesized(fld.fold_parenthesized_parameter_data(data)),
    }
}

pub fn noop_fold_angle_bracketed_parameter_data<T: Folder>(data: AngleBracketedParameterData,
                                                           fld: &mut T)
                                                           -> AngleBracketedParameterData
{
    let AngleBracketedParameterData { lifetimes, types, bindings } = data;
    AngleBracketedParameterData { lifetimes: fld.fold_lifetimes(lifetimes),
                                  types: types.move_map(|ty| fld.fold_ty(ty)),
                                  bindings: bindings.move_map(|b| fld.fold_ty_binding(b)) }
}

pub fn noop_fold_parenthesized_parameter_data<T: Folder>(data: ParenthesizedParameterData,
                                                         fld: &mut T)
                                                         -> ParenthesizedParameterData
{
    let ParenthesizedParameterData { inputs, output, span } = data;
    ParenthesizedParameterData { inputs: inputs.move_map(|ty| fld.fold_ty(ty)),
                                 output: output.map(|ty| fld.fold_ty(ty)),
                                 span: fld.new_span(span) }
}

pub fn noop_fold_local<T: Folder>(l: P<Local>, fld: &mut T) -> P<Local> {
    l.map(|Local {id, pat, ty, init, span, attrs}| Local {
        id: fld.new_id(id),
        ty: ty.map(|t| fld.fold_ty(t)),
        pat: fld.fold_pat(pat),
        init: init.map(|e| fld.fold_expr(e)),
        span: fld.new_span(span),
        attrs: attrs.map_thin_attrs(|v| fold_attrs(v, fld)),
    })
}

pub fn noop_fold_attribute<T: Folder>(at: Attribute, fld: &mut T) -> Option<Attribute> {
    let Spanned {node: Attribute_ {id, style, value, is_sugared_doc}, span} = at;
    Some(Spanned {
        node: Attribute_ {
            id: id,
            style: style,
            value: fld.fold_meta_item(value),
            is_sugared_doc: is_sugared_doc
        },
        span: fld.new_span(span)
    })
}

pub fn noop_fold_explicit_self_kind<T: Folder>(es: SelfKind, fld: &mut T)
                                                     -> SelfKind {
    match es {
        SelfKind::Static | SelfKind::Value(_) => es,
        SelfKind::Region(lifetime, m, ident) => {
            SelfKind::Region(fld.fold_opt_lifetime(lifetime), m, ident)
        }
        SelfKind::Explicit(typ, ident) => {
            SelfKind::Explicit(fld.fold_ty(typ), ident)
        }
    }
}

pub fn noop_fold_explicit_self<T: Folder>(Spanned {span, node}: ExplicitSelf, fld: &mut T)
                                          -> ExplicitSelf {
    Spanned {
        node: fld.fold_explicit_self_kind(node),
        span: fld.new_span(span)
    }
}


pub fn noop_fold_mac<T: Folder>(Spanned {node, span}: Mac, fld: &mut T) -> Mac {
    Spanned {
        node: Mac_ {
            path: fld.fold_path(node.path),
            tts: fld.fold_tts(&node.tts),
            ctxt: node.ctxt,
        },
        span: fld.new_span(span)
    }
}

pub fn noop_fold_meta_item<T: Folder>(mi: P<MetaItem>, fld: &mut T) -> P<MetaItem> {
    mi.map(|Spanned {node, span}| Spanned {
        node: match node {
            MetaItemKind::Word(id) => MetaItemKind::Word(id),
            MetaItemKind::List(id, mis) => {
                MetaItemKind::List(id, mis.move_map(|e| fld.fold_meta_item(e)))
            }
            MetaItemKind::NameValue(id, s) => MetaItemKind::NameValue(id, s)
        },
        span: fld.new_span(span)
    })
}

pub fn noop_fold_arg<T: Folder>(Arg {id, pat, ty}: Arg, fld: &mut T) -> Arg {
    Arg {
        id: fld.new_id(id),
        pat: fld.fold_pat(pat),
        ty: fld.fold_ty(ty)
    }
}

pub fn noop_fold_tt<T: Folder>(tt: &TokenTree, fld: &mut T) -> TokenTree {
    match *tt {
        TokenTree::Token(span, ref tok) =>
            TokenTree::Token(span, fld.fold_token(tok.clone())),
        TokenTree::Delimited(span, ref delimed) => {
            TokenTree::Delimited(span, Rc::new(
                            Delimited {
                                delim: delimed.delim,
                                open_span: delimed.open_span,
                                tts: fld.fold_tts(&delimed.tts),
                                close_span: delimed.close_span,
                            }
                        ))
        },
        TokenTree::Sequence(span, ref seq) =>
            TokenTree::Sequence(span,
                       Rc::new(SequenceRepetition {
                           tts: fld.fold_tts(&seq.tts),
                           separator: seq.separator.clone().map(|tok| fld.fold_token(tok)),
                           ..**seq
                       })),
    }
}

pub fn noop_fold_tts<T: Folder>(tts: &[TokenTree], fld: &mut T) -> Vec<TokenTree> {
    // FIXME: Does this have to take a tts slice?
    // Could use move_map otherwise...
    tts.iter().map(|tt| fld.fold_tt(tt)).collect()
}

// apply ident folder if it's an ident, apply other folds to interpolated nodes
pub fn noop_fold_token<T: Folder>(t: token::Token, fld: &mut T) -> token::Token {
    match t {
        token::Ident(id, followed_by_colons) => {
            token::Ident(fld.fold_ident(id), followed_by_colons)
        }
        token::Lifetime(id) => token::Lifetime(fld.fold_ident(id)),
        token::Interpolated(nt) => token::Interpolated(fld.fold_interpolated(nt)),
        token::SubstNt(ident, namep) => {
            token::SubstNt(fld.fold_ident(ident), namep)
        }
        token::MatchNt(name, kind, namep, kindp) => {
            token::MatchNt(fld.fold_ident(name), fld.fold_ident(kind), namep, kindp)
        }
        _ => t
    }
}

/// apply folder to elements of interpolated nodes
//
// NB: this can occur only when applying a fold to partially expanded code, where
// parsed pieces have gotten implanted ito *other* macro invocations. This is relevant
// for macro hygiene, but possibly not elsewhere.
//
// One problem here occurs because the types for fold_item, fold_stmt, etc. allow the
// folder to return *multiple* items; this is a problem for the nodes here, because
// they insist on having exactly one piece. One solution would be to mangle the fold
// trait to include one-to-many and one-to-one versions of these entry points, but that
// would probably confuse a lot of people and help very few. Instead, I'm just going
// to put in dynamic checks. I think the performance impact of this will be pretty much
// nonexistent. The danger is that someone will apply a fold to a partially expanded
// node, and will be confused by the fact that their "fold_item" or "fold_stmt" isn't
// getting called on NtItem or NtStmt nodes. Hopefully they'll wind up reading this
// comment, and doing something appropriate.
//
// BTW, design choice: I considered just changing the type of, e.g., NtItem to contain
// multiple items, but decided against it when I looked at parse_item_or_view_item and
// tried to figure out what I would do with multiple items there....
pub fn noop_fold_interpolated<T: Folder>(nt: token::Nonterminal, fld: &mut T)
                                         -> token::Nonterminal {
    match nt {
        token::NtItem(item) =>
            token::NtItem(fld.fold_item(item)
                          // this is probably okay, because the only folds likely
                          // to peek inside interpolated nodes will be renamings/markings,
                          // which map single items to single items
                          .expect_one("expected fold to produce exactly one item")),
        token::NtBlock(block) => token::NtBlock(fld.fold_block(block)),
        token::NtStmt(stmt) =>
            token::NtStmt(stmt.map(|stmt| fld.fold_stmt(stmt)
                          // this is probably okay, because the only folds likely
                          // to peek inside interpolated nodes will be renamings/markings,
                          // which map single items to single items
                          .expect_one("expected fold to produce exactly one statement"))),
        token::NtPat(pat) => token::NtPat(fld.fold_pat(pat)),
        token::NtExpr(expr) => token::NtExpr(fld.fold_expr(expr)),
        token::NtTy(ty) => token::NtTy(fld.fold_ty(ty)),
        token::NtIdent(id, is_mod_name) =>
            token::NtIdent(Box::new(Spanned::<Ident>{node: fld.fold_ident(id.node), .. *id}),
                           is_mod_name),
        token::NtMeta(meta_item) => token::NtMeta(fld.fold_meta_item(meta_item)),
        token::NtPath(path) => token::NtPath(Box::new(fld.fold_path(*path))),
        token::NtTT(tt) => token::NtTT(P(fld.fold_tt(&tt))),
        token::NtArm(arm) => token::NtArm(fld.fold_arm(arm)),
        token::NtImplItem(arm) =>
            token::NtImplItem(arm.map(|arm| fld.fold_impl_item(arm)
                              .expect_one("expected fold to produce exactly one item"))),
        token::NtTraitItem(arm) =>
            token::NtTraitItem(arm.map(|arm| fld.fold_trait_item(arm)
                               .expect_one("expected fold to produce exactly one item"))),
        token::NtGenerics(generics) => token::NtGenerics(fld.fold_generics(generics)),
        token::NtWhereClause(where_clause) =>
            token::NtWhereClause(fld.fold_where_clause(where_clause)),
        token::NtArg(arg) => token::NtArg(fld.fold_arg(arg)),
    }
}

pub fn noop_fold_fn_decl<T: Folder>(decl: P<FnDecl>, fld: &mut T) -> P<FnDecl> {
    decl.map(|FnDecl {inputs, output, variadic}| FnDecl {
        inputs: inputs.move_map(|x| fld.fold_arg(x)),
        output: match output {
            FunctionRetTy::Ty(ty) => FunctionRetTy::Ty(fld.fold_ty(ty)),
            FunctionRetTy::Default(span) => FunctionRetTy::Default(span),
            FunctionRetTy::None(span) => FunctionRetTy::None(span),
        },
        variadic: variadic
    })
}

pub fn noop_fold_ty_param_bound<T>(tpb: TyParamBound, fld: &mut T)
                                   -> TyParamBound
                                   where T: Folder {
    match tpb {
        TraitTyParamBound(ty, modifier) => TraitTyParamBound(fld.fold_poly_trait_ref(ty), modifier),
        RegionTyParamBound(lifetime) => RegionTyParamBound(fld.fold_lifetime(lifetime)),
    }
}

pub fn noop_fold_ty_param<T: Folder>(tp: TyParam, fld: &mut T) -> TyParam {
    let TyParam {id, ident, bounds, default, span} = tp;
    TyParam {
        id: fld.new_id(id),
        ident: ident,
        bounds: fld.fold_bounds(bounds),
        default: default.map(|x| fld.fold_ty(x)),
        span: span
    }
}

pub fn noop_fold_ty_params<T: Folder>(tps: P<[TyParam]>, fld: &mut T)
                                      -> P<[TyParam]> {
    tps.move_map(|tp| fld.fold_ty_param(tp))
}

pub fn noop_fold_lifetime<T: Folder>(l: Lifetime, fld: &mut T) -> Lifetime {
    Lifetime {
        id: fld.new_id(l.id),
        name: l.name,
        span: fld.new_span(l.span)
    }
}

pub fn noop_fold_lifetime_def<T: Folder>(l: LifetimeDef, fld: &mut T)
                                         -> LifetimeDef {
    LifetimeDef {
        lifetime: fld.fold_lifetime(l.lifetime),
        bounds: fld.fold_lifetimes(l.bounds),
    }
}

pub fn noop_fold_lifetimes<T: Folder>(lts: Vec<Lifetime>, fld: &mut T) -> Vec<Lifetime> {
    lts.move_map(|l| fld.fold_lifetime(l))
}

pub fn noop_fold_lifetime_defs<T: Folder>(lts: Vec<LifetimeDef>, fld: &mut T)
                                          -> Vec<LifetimeDef> {
    lts.move_map(|l| fld.fold_lifetime_def(l))
}

pub fn noop_fold_opt_lifetime<T: Folder>(o_lt: Option<Lifetime>, fld: &mut T)
                                         -> Option<Lifetime> {
    o_lt.map(|lt| fld.fold_lifetime(lt))
}

pub fn noop_fold_generics<T: Folder>(Generics {ty_params, lifetimes, where_clause}: Generics,
                                     fld: &mut T) -> Generics {
    Generics {
        ty_params: fld.fold_ty_params(ty_params),
        lifetimes: fld.fold_lifetime_defs(lifetimes),
        where_clause: fld.fold_where_clause(where_clause),
    }
}

pub fn noop_fold_where_clause<T: Folder>(
                              WhereClause {id, predicates}: WhereClause,
                              fld: &mut T)
                              -> WhereClause {
    WhereClause {
        id: fld.new_id(id),
        predicates: predicates.move_map(|predicate| {
            fld.fold_where_predicate(predicate)
        })
    }
}

pub fn noop_fold_where_predicate<T: Folder>(
                                 pred: WherePredicate,
                                 fld: &mut T)
                                 -> WherePredicate {
    match pred {
        ast::WherePredicate::BoundPredicate(ast::WhereBoundPredicate{bound_lifetimes,
                                                                     bounded_ty,
                                                                     bounds,
                                                                     span}) => {
            ast::WherePredicate::BoundPredicate(ast::WhereBoundPredicate {
                bound_lifetimes: fld.fold_lifetime_defs(bound_lifetimes),
                bounded_ty: fld.fold_ty(bounded_ty),
                bounds: bounds.move_map(|x| fld.fold_ty_param_bound(x)),
                span: fld.new_span(span)
            })
        }
        ast::WherePredicate::RegionPredicate(ast::WhereRegionPredicate{lifetime,
                                                                       bounds,
                                                                       span}) => {
            ast::WherePredicate::RegionPredicate(ast::WhereRegionPredicate {
                span: fld.new_span(span),
                lifetime: fld.fold_lifetime(lifetime),
                bounds: bounds.move_map(|bound| fld.fold_lifetime(bound))
            })
        }
        ast::WherePredicate::EqPredicate(ast::WhereEqPredicate{id,
                                                               path,
                                                               ty,
                                                               span}) => {
            ast::WherePredicate::EqPredicate(ast::WhereEqPredicate{
                id: fld.new_id(id),
                path: fld.fold_path(path),
                ty:fld.fold_ty(ty),
                span: fld.new_span(span)
            })
        }
    }
}

pub fn noop_fold_variant_data<T: Folder>(vdata: VariantData, fld: &mut T) -> VariantData {
    match vdata {
        ast::VariantData::Struct(fields, id) => {
            ast::VariantData::Struct(fields.move_map(|f| fld.fold_struct_field(f)),
                                     fld.new_id(id))
        }
        ast::VariantData::Tuple(fields, id) => {
            ast::VariantData::Tuple(fields.move_map(|f| fld.fold_struct_field(f)),
                                    fld.new_id(id))
        }
        ast::VariantData::Unit(id) => ast::VariantData::Unit(fld.new_id(id))
    }
}

pub fn noop_fold_trait_ref<T: Folder>(p: TraitRef, fld: &mut T) -> TraitRef {
    let id = fld.new_id(p.ref_id);
    let TraitRef {
        path,
        ref_id: _,
    } = p;
    ast::TraitRef {
        path: fld.fold_path(path),
        ref_id: id,
    }
}

pub fn noop_fold_poly_trait_ref<T: Folder>(p: PolyTraitRef, fld: &mut T) -> PolyTraitRef {
    ast::PolyTraitRef {
        bound_lifetimes: fld.fold_lifetime_defs(p.bound_lifetimes),
        trait_ref: fld.fold_trait_ref(p.trait_ref),
        span: fld.new_span(p.span),
    }
}

pub fn noop_fold_struct_field<T: Folder>(f: StructField, fld: &mut T) -> StructField {
    let StructField {node: StructField_ {id, kind, ty, attrs}, span} = f;
    Spanned {
        node: StructField_ {
            id: fld.new_id(id),
            kind: kind,
            ty: fld.fold_ty(ty),
            attrs: fold_attrs(attrs, fld),
        },
        span: fld.new_span(span)
    }
}

pub fn noop_fold_field<T: Folder>(Field {ident, expr, span}: Field, folder: &mut T) -> Field {
    Field {
        ident: respan(ident.span, folder.fold_ident(ident.node)),
        expr: folder.fold_expr(expr),
        span: folder.new_span(span)
    }
}

pub fn noop_fold_mt<T: Folder>(MutTy {ty, mutbl}: MutTy, folder: &mut T) -> MutTy {
    MutTy {
        ty: folder.fold_ty(ty),
        mutbl: mutbl,
    }
}

pub fn noop_fold_opt_bounds<T: Folder>(b: Option<TyParamBounds>, folder: &mut T)
                                       -> Option<TyParamBounds> {
    b.map(|bounds| folder.fold_bounds(bounds))
}

fn noop_fold_bounds<T: Folder>(bounds: TyParamBounds, folder: &mut T)
                          -> TyParamBounds {
    bounds.move_map(|bound| folder.fold_ty_param_bound(bound))
}

pub fn noop_fold_block<T: Folder>(b: P<Block>, folder: &mut T) -> P<Block> {
    b.map(|Block {id, stmts, expr, rules, span}| Block {
        id: folder.new_id(id),
        stmts: stmts.move_flat_map(|s| folder.fold_stmt(s).into_iter()),
        expr: expr.and_then(|x| folder.fold_opt_expr(x)),
        rules: rules,
        span: folder.new_span(span),
    })
}

pub fn noop_fold_item_kind<T: Folder>(i: ItemKind, folder: &mut T) -> ItemKind {
    match i {
        ItemKind::ExternCrate(string) => ItemKind::ExternCrate(string),
        ItemKind::Use(view_path) => {
            ItemKind::Use(folder.fold_view_path(view_path))
        }
        ItemKind::Static(t, m, e) => {
            ItemKind::Static(folder.fold_ty(t), m, folder.fold_expr(e))
        }
        ItemKind::Const(t, e) => {
            ItemKind::Const(folder.fold_ty(t), folder.fold_expr(e))
        }
        ItemKind::Fn(decl, unsafety, constness, abi, generics, body) => {
            ItemKind::Fn(
                folder.fold_fn_decl(decl),
                unsafety,
                constness,
                abi,
                folder.fold_generics(generics),
                folder.fold_block(body)
            )
        }
        ItemKind::Mod(m) => ItemKind::Mod(folder.fold_mod(m)),
        ItemKind::ForeignMod(nm) => ItemKind::ForeignMod(folder.fold_foreign_mod(nm)),
        ItemKind::Ty(t, generics) => {
            ItemKind::Ty(folder.fold_ty(t), folder.fold_generics(generics))
        }
        ItemKind::Enum(enum_definition, generics) => {
            ItemKind::Enum(
                ast::EnumDef {
                    variants: enum_definition.variants.move_map(|x| folder.fold_variant(x)),
                },
                folder.fold_generics(generics))
        }
        ItemKind::Struct(struct_def, generics) => {
            let struct_def = folder.fold_variant_data(struct_def);
            ItemKind::Struct(struct_def, folder.fold_generics(generics))
        }
        ItemKind::DefaultImpl(unsafety, ref trait_ref) => {
            ItemKind::DefaultImpl(unsafety, folder.fold_trait_ref((*trait_ref).clone()))
        }
        ItemKind::Impl(unsafety, polarity, generics, ifce, ty, impl_items) => {
            let new_impl_items = impl_items.move_flat_map(|item| {
                folder.fold_impl_item(item)
            });
            let ifce = match ifce {
                None => None,
                Some(ref trait_ref) => {
                    Some(folder.fold_trait_ref((*trait_ref).clone()))
                }
            };
            ItemKind::Impl(unsafety,
                     polarity,
                     folder.fold_generics(generics),
                     ifce,
                     folder.fold_ty(ty),
                     new_impl_items)
        }
        ItemKind::Trait(unsafety, generics, bounds, items) => {
            let bounds = folder.fold_bounds(bounds);
            let items = items.move_flat_map(|item| {
                folder.fold_trait_item(item)
            });
            ItemKind::Trait(unsafety,
                      folder.fold_generics(generics),
                      bounds,
                      items)
        }
        ItemKind::Mac(m) => ItemKind::Mac(folder.fold_mac(m)),
    }
}

pub fn noop_fold_trait_item<T: Folder>(i: TraitItem, folder: &mut T)
                                       -> SmallVector<TraitItem> {
    SmallVector::one(TraitItem {
        id: folder.new_id(i.id),
        ident: folder.fold_ident(i.ident),
        attrs: fold_attrs(i.attrs, folder),
        node: match i.node {
            TraitItemKind::Const(ty, default) => {
                TraitItemKind::Const(folder.fold_ty(ty),
                               default.map(|x| folder.fold_expr(x)))
            }
            TraitItemKind::Method(sig, body) => {
                TraitItemKind::Method(noop_fold_method_sig(sig, folder),
                                body.map(|x| folder.fold_block(x)))
            }
            TraitItemKind::Type(bounds, default) => {
                TraitItemKind::Type(folder.fold_bounds(bounds),
                              default.map(|x| folder.fold_ty(x)))
            }
        },
        span: folder.new_span(i.span)
    })
}

pub fn noop_fold_impl_item<T: Folder>(i: ImplItem, folder: &mut T)
                                      -> SmallVector<ImplItem> {
    SmallVector::one(ImplItem {
        id: folder.new_id(i.id),
        ident: folder.fold_ident(i.ident),
        attrs: fold_attrs(i.attrs, folder),
        vis: i.vis,
        defaultness: i.defaultness,
        node: match i.node  {
            ast::ImplItemKind::Const(ty, expr) => {
                ast::ImplItemKind::Const(folder.fold_ty(ty), folder.fold_expr(expr))
            }
            ast::ImplItemKind::Method(sig, body) => {
                ast::ImplItemKind::Method(noop_fold_method_sig(sig, folder),
                               folder.fold_block(body))
            }
            ast::ImplItemKind::Type(ty) => ast::ImplItemKind::Type(folder.fold_ty(ty)),
            ast::ImplItemKind::Macro(mac) => ast::ImplItemKind::Macro(folder.fold_mac(mac))
        },
        span: folder.new_span(i.span)
    })
}

pub fn noop_fold_mod<T: Folder>(Mod {inner, items}: Mod, folder: &mut T) -> Mod {
    Mod {
        inner: folder.new_span(inner),
        items: items.move_flat_map(|x| folder.fold_item(x)),
    }
}

pub fn noop_fold_crate<T: Folder>(Crate {module, attrs, config, mut exported_macros, span}: Crate,
                                  folder: &mut T) -> Crate {
    let config = folder.fold_meta_items(config);

    let mut items = folder.fold_item(P(ast::Item {
        ident: token::special_idents::invalid,
        attrs: attrs,
        id: ast::DUMMY_NODE_ID,
        vis: ast::Visibility::Public,
        span: span,
        node: ast::ItemKind::Mod(module),
    })).into_iter();

    let (module, attrs, span) = match items.next() {
        Some(item) => {
            assert!(items.next().is_none(),
                    "a crate cannot expand to more than one item");
            item.and_then(|ast::Item { attrs, span, node, .. }| {
                match node {
                    ast::ItemKind::Mod(m) => (m, attrs, span),
                    _ => panic!("fold converted a module to not a module"),
                }
            })
        }
        None => (ast::Mod {
            inner: span,
            items: vec![],
        }, vec![], span)
    };

    for def in &mut exported_macros {
        def.id = folder.new_id(def.id);
    }

    Crate {
        module: module,
        attrs: attrs,
        config: config,
        exported_macros: exported_macros,
        span: span,
    }
}

// fold one item into possibly many items
pub fn noop_fold_item<T: Folder>(i: P<Item>, folder: &mut T) -> SmallVector<P<Item>> {
    SmallVector::one(i.map(|i| folder.fold_item_simple(i)))
}

// fold one item into exactly one item
pub fn noop_fold_item_simple<T: Folder>(Item {id, ident, attrs, node, vis, span}: Item,
                                        folder: &mut T) -> Item {
    let id = folder.new_id(id);
    let node = folder.fold_item_kind(node);
    let ident = match node {
        // The node may have changed, recompute the "pretty" impl name.
        ItemKind::Impl(_, _, _, ref maybe_trait, ref ty, _) => {
            ast_util::impl_pretty_name(maybe_trait, Some(&ty))
        }
        _ => ident
    };

    Item {
        id: id,
        ident: folder.fold_ident(ident),
        attrs: fold_attrs(attrs, folder),
        node: node,
        vis: vis,
        span: folder.new_span(span)
    }
}

pub fn noop_fold_foreign_item<T: Folder>(ni: ForeignItem, folder: &mut T) -> ForeignItem {
    ForeignItem {
        id: folder.new_id(ni.id),
        ident: folder.fold_ident(ni.ident),
        attrs: fold_attrs(ni.attrs, folder),
        node: match ni.node {
            ForeignItemKind::Fn(fdec, generics) => {
                ForeignItemKind::Fn(folder.fold_fn_decl(fdec), folder.fold_generics(generics))
            }
            ForeignItemKind::Static(t, m) => {
                ForeignItemKind::Static(folder.fold_ty(t), m)
            }
        },
        vis: ni.vis,
        span: folder.new_span(ni.span)
    }
}

pub fn noop_fold_method_sig<T: Folder>(sig: MethodSig, folder: &mut T) -> MethodSig {
    MethodSig {
        generics: folder.fold_generics(sig.generics),
        abi: sig.abi,
        explicit_self: folder.fold_explicit_self(sig.explicit_self),
        unsafety: sig.unsafety,
        constness: sig.constness,
        decl: folder.fold_fn_decl(sig.decl)
    }
}

pub fn noop_fold_pat<T: Folder>(p: P<Pat>, folder: &mut T) -> P<Pat> {
    p.map(|Pat {id, node, span}| Pat {
        id: folder.new_id(id),
        node: match node {
            PatKind::Wild => PatKind::Wild,
            PatKind::Ident(binding_mode, pth1, sub) => {
                PatKind::Ident(binding_mode,
                        Spanned{span: folder.new_span(pth1.span),
                                node: folder.fold_ident(pth1.node)},
                        sub.map(|x| folder.fold_pat(x)))
            }
            PatKind::Lit(e) => PatKind::Lit(folder.fold_expr(e)),
            PatKind::TupleStruct(pth, pats) => {
                PatKind::TupleStruct(folder.fold_path(pth),
                        pats.map(|pats| pats.move_map(|x| folder.fold_pat(x))))
            }
            PatKind::Path(pth) => {
                PatKind::Path(folder.fold_path(pth))
            }
            PatKind::QPath(qself, pth) => {
                let qself = QSelf {ty: folder.fold_ty(qself.ty), .. qself};
                PatKind::QPath(qself, folder.fold_path(pth))
            }
            PatKind::Struct(pth, fields, etc) => {
                let pth = folder.fold_path(pth);
                let fs = fields.move_map(|f| {
                    Spanned { span: folder.new_span(f.span),
                              node: ast::FieldPat {
                                  ident: f.node.ident,
                                  pat: folder.fold_pat(f.node.pat),
                                  is_shorthand: f.node.is_shorthand,
                              }}
                });
                PatKind::Struct(pth, fs, etc)
            }
            PatKind::Tup(elts) => PatKind::Tup(elts.move_map(|x| folder.fold_pat(x))),
            PatKind::Box(inner) => PatKind::Box(folder.fold_pat(inner)),
            PatKind::Ref(inner, mutbl) => PatKind::Ref(folder.fold_pat(inner), mutbl),
            PatKind::Range(e1, e2) => {
                PatKind::Range(folder.fold_expr(e1), folder.fold_expr(e2))
            },
            PatKind::Vec(before, slice, after) => {
                PatKind::Vec(before.move_map(|x| folder.fold_pat(x)),
                       slice.map(|x| folder.fold_pat(x)),
                       after.move_map(|x| folder.fold_pat(x)))
            }
            PatKind::Mac(mac) => PatKind::Mac(folder.fold_mac(mac))
        },
        span: folder.new_span(span)
    })
}

pub fn noop_fold_expr<T: Folder>(Expr {id, node, span, attrs}: Expr, folder: &mut T) -> Expr {
    Expr {
        id: folder.new_id(id),
        node: match node {
            ExprKind::Box(e) => {
                ExprKind::Box(folder.fold_expr(e))
            }
            ExprKind::InPlace(p, e) => {
                ExprKind::InPlace(folder.fold_expr(p), folder.fold_expr(e))
            }
            ExprKind::Vec(exprs) => {
                ExprKind::Vec(folder.fold_exprs(exprs))
            }
            ExprKind::Repeat(expr, count) => {
                ExprKind::Repeat(folder.fold_expr(expr), folder.fold_expr(count))
            }
            ExprKind::Tup(exprs) => ExprKind::Tup(folder.fold_exprs(exprs)),
            ExprKind::Call(f, args) => {
                ExprKind::Call(folder.fold_expr(f),
                         folder.fold_exprs(args))
            }
            ExprKind::MethodCall(i, tps, args) => {
                ExprKind::MethodCall(
                    respan(folder.new_span(i.span), folder.fold_ident(i.node)),
                    tps.move_map(|x| folder.fold_ty(x)),
                    folder.fold_exprs(args))
            }
            ExprKind::Binary(binop, lhs, rhs) => {
                ExprKind::Binary(binop,
                        folder.fold_expr(lhs),
                        folder.fold_expr(rhs))
            }
            ExprKind::Unary(binop, ohs) => {
                ExprKind::Unary(binop, folder.fold_expr(ohs))
            }
            ExprKind::Lit(l) => ExprKind::Lit(l),
            ExprKind::Cast(expr, ty) => {
                ExprKind::Cast(folder.fold_expr(expr), folder.fold_ty(ty))
            }
            ExprKind::Type(expr, ty) => {
                ExprKind::Type(folder.fold_expr(expr), folder.fold_ty(ty))
            }
            ExprKind::AddrOf(m, ohs) => ExprKind::AddrOf(m, folder.fold_expr(ohs)),
            ExprKind::If(cond, tr, fl) => {
                ExprKind::If(folder.fold_expr(cond),
                       folder.fold_block(tr),
                       fl.map(|x| folder.fold_expr(x)))
            }
            ExprKind::IfLet(pat, expr, tr, fl) => {
                ExprKind::IfLet(folder.fold_pat(pat),
                          folder.fold_expr(expr),
                          folder.fold_block(tr),
                          fl.map(|x| folder.fold_expr(x)))
            }
            ExprKind::While(cond, body, opt_ident) => {
                ExprKind::While(folder.fold_expr(cond),
                          folder.fold_block(body),
                          opt_ident.map(|i| folder.fold_ident(i)))
            }
            ExprKind::WhileLet(pat, expr, body, opt_ident) => {
                ExprKind::WhileLet(folder.fold_pat(pat),
                             folder.fold_expr(expr),
                             folder.fold_block(body),
                             opt_ident.map(|i| folder.fold_ident(i)))
            }
            ExprKind::ForLoop(pat, iter, body, opt_ident) => {
                ExprKind::ForLoop(folder.fold_pat(pat),
                            folder.fold_expr(iter),
                            folder.fold_block(body),
                            opt_ident.map(|i| folder.fold_ident(i)))
            }
            ExprKind::Loop(body, opt_ident) => {
                ExprKind::Loop(folder.fold_block(body),
                        opt_ident.map(|i| folder.fold_ident(i)))
            }
            ExprKind::Match(expr, arms) => {
                ExprKind::Match(folder.fold_expr(expr),
                          arms.move_map(|x| folder.fold_arm(x)))
            }
            ExprKind::Closure(capture_clause, decl, body) => {
                ExprKind::Closure(capture_clause,
                            folder.fold_fn_decl(decl),
                            folder.fold_block(body))
            }
            ExprKind::Block(blk) => ExprKind::Block(folder.fold_block(blk)),
            ExprKind::Assign(el, er) => {
                ExprKind::Assign(folder.fold_expr(el), folder.fold_expr(er))
            }
            ExprKind::AssignOp(op, el, er) => {
                ExprKind::AssignOp(op,
                            folder.fold_expr(el),
                            folder.fold_expr(er))
            }
            ExprKind::Field(el, ident) => {
                ExprKind::Field(folder.fold_expr(el),
                          respan(folder.new_span(ident.span),
                                 folder.fold_ident(ident.node)))
            }
            ExprKind::TupField(el, ident) => {
                ExprKind::TupField(folder.fold_expr(el),
                             respan(folder.new_span(ident.span),
                                    folder.fold_usize(ident.node)))
            }
            ExprKind::Index(el, er) => {
                ExprKind::Index(folder.fold_expr(el), folder.fold_expr(er))
            }
            ExprKind::Range(e1, e2, lim) => {
                ExprKind::Range(e1.map(|x| folder.fold_expr(x)),
                                e2.map(|x| folder.fold_expr(x)),
                                lim)
            }
            ExprKind::Path(qself, path) => {
                let qself = qself.map(|QSelf { ty, position }| {
                    QSelf {
                        ty: folder.fold_ty(ty),
                        position: position
                    }
                });
                ExprKind::Path(qself, folder.fold_path(path))
            }
            ExprKind::Break(opt_ident) => ExprKind::Break(opt_ident.map(|label|
                respan(folder.new_span(label.span),
                       folder.fold_ident(label.node)))
            ),
            ExprKind::Again(opt_ident) => ExprKind::Again(opt_ident.map(|label|
                respan(folder.new_span(label.span),
                       folder.fold_ident(label.node)))
            ),
            ExprKind::Ret(e) => ExprKind::Ret(e.map(|x| folder.fold_expr(x))),
            ExprKind::InlineAsm(InlineAsm {
                inputs,
                outputs,
                asm,
                asm_str_style,
                clobbers,
                volatile,
                alignstack,
                dialect,
                expn_id,
            }) => ExprKind::InlineAsm(InlineAsm {
                inputs: inputs.move_map(|(c, input)| {
                    (c, folder.fold_expr(input))
                }),
                outputs: outputs.move_map(|out| {
                    InlineAsmOutput {
                        constraint: out.constraint,
                        expr: folder.fold_expr(out.expr),
                        is_rw: out.is_rw,
                        is_indirect: out.is_indirect,
                    }
                }),
                asm: asm,
                asm_str_style: asm_str_style,
                clobbers: clobbers,
                volatile: volatile,
                alignstack: alignstack,
                dialect: dialect,
                expn_id: expn_id,
            }),
            ExprKind::Mac(mac) => ExprKind::Mac(folder.fold_mac(mac)),
            ExprKind::Struct(path, fields, maybe_expr) => {
                ExprKind::Struct(folder.fold_path(path),
                        fields.move_map(|x| folder.fold_field(x)),
                        maybe_expr.map(|x| folder.fold_expr(x)))
            },
            ExprKind::Paren(ex) => ExprKind::Paren(folder.fold_expr(ex)),
            ExprKind::Try(ex) => ExprKind::Try(folder.fold_expr(ex)),
        },
        span: folder.new_span(span),
        attrs: attrs.map_thin_attrs(|v| fold_attrs(v, folder)),
    }
}

pub fn noop_fold_opt_expr<T: Folder>(e: P<Expr>, folder: &mut T) -> Option<P<Expr>> {
    Some(folder.fold_expr(e))
}

pub fn noop_fold_exprs<T: Folder>(es: Vec<P<Expr>>, folder: &mut T) -> Vec<P<Expr>> {
    es.move_flat_map(|e| folder.fold_opt_expr(e))
}

pub fn noop_fold_stmt<T: Folder>(Spanned {node, span}: Stmt, folder: &mut T)
                                 -> SmallVector<Stmt> {
    let span = folder.new_span(span);
    match node {
        StmtKind::Decl(d, id) => {
            let id = folder.new_id(id);
            folder.fold_decl(d).into_iter().map(|d| Spanned {
                node: StmtKind::Decl(d, id),
                span: span
            }).collect()
        }
        StmtKind::Expr(e, id) => {
            let id = folder.new_id(id);
            if let Some(e) = folder.fold_opt_expr(e) {
                SmallVector::one(Spanned {
                    node: StmtKind::Expr(e, id),
                    span: span
                })
            } else {
                SmallVector::zero()
            }
        }
        StmtKind::Semi(e, id) => {
            let id = folder.new_id(id);
            if let Some(e) = folder.fold_opt_expr(e) {
                SmallVector::one(Spanned {
                    node: StmtKind::Semi(e, id),
                    span: span
                })
            } else {
                SmallVector::zero()
            }
        }
        StmtKind::Mac(mac, semi, attrs) => SmallVector::one(Spanned {
            node: StmtKind::Mac(mac.map(|m| folder.fold_mac(m)),
                                semi,
                                attrs.map_thin_attrs(|v| fold_attrs(v, folder))),
            span: span
        })
    }
}

#[cfg(test)]
mod tests {
    use std::io;
    use ast;
    use util::parser_testing::{string_to_crate, matches_codepattern};
    use parse::token;
    use print::pprust;
    use fold;
    use super::*;

    // this version doesn't care about getting comments or docstrings in.
    fn fake_print_crate(s: &mut pprust::State,
                        krate: &ast::Crate) -> io::Result<()> {
        s.print_mod(&krate.module, &krate.attrs)
    }

    // change every identifier to "zz"
    struct ToZzIdentFolder;

    impl Folder for ToZzIdentFolder {
        fn fold_ident(&mut self, _: ast::Ident) -> ast::Ident {
            token::str_to_ident("zz")
        }
        fn fold_mac(&mut self, mac: ast::Mac) -> ast::Mac {
            fold::noop_fold_mac(mac, self)
        }
    }

    // maybe add to expand.rs...
    macro_rules! assert_pred {
        ($pred:expr, $predname:expr, $a:expr , $b:expr) => (
            {
                let pred_val = $pred;
                let a_val = $a;
                let b_val = $b;
                if !(pred_val(&a_val, &b_val)) {
                    panic!("expected args satisfying {}, got {} and {}",
                          $predname, a_val, b_val);
                }
            }
        )
    }

    // make sure idents get transformed everywhere
    #[test] fn ident_transformation () {
        let mut zz_fold = ToZzIdentFolder;
        let ast = string_to_crate(
            "#[a] mod b {fn c (d : e, f : g) {h!(i,j,k);l;m}}".to_string());
        let folded_crate = zz_fold.fold_crate(ast);
        assert_pred!(
            matches_codepattern,
            "matches_codepattern",
            pprust::to_string(|s| fake_print_crate(s, &folded_crate)),
            "#[a]mod zz{fn zz(zz:zz,zz:zz){zz!(zz,zz,zz);zz;zz}}".to_string());
    }

    // even inside macro defs....
    #[test] fn ident_transformation_in_defs () {
        let mut zz_fold = ToZzIdentFolder;
        let ast = string_to_crate(
            "macro_rules! a {(b $c:expr $(d $e:token)f+ => \
             (g $(d $d $e)+))} ".to_string());
        let folded_crate = zz_fold.fold_crate(ast);
        assert_pred!(
            matches_codepattern,
            "matches_codepattern",
            pprust::to_string(|s| fake_print_crate(s, &folded_crate)),
            "zz!zz((zz$zz:zz$(zz $zz:zz)zz+=>(zz$(zz$zz$zz)+)));".to_string());
    }
}
