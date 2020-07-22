use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;

mod parse;

use parse::MatcherDerive;

#[proc_macro_derive(Matcher, attributes(matcher))]
pub fn derive(input: TokenStream) -> TokenStream {
    // TODO when we generate a name that isn't a valid ident or is a keyword, generate a different
    // name rather than panicking.
    // TODO handle generics in the input
    let input = parse_macro_input!(input as MatcherDerive);

    let enum_name = &input.enum_name;
    let matcher_name = input.resolve_matcher_name();

    let struct_fields = input.variants.iter().map(|variant| {
        let builder_name = variant.resolve_builder_name();
        let variant_ty = &variant.field.ty;
        quote!(#builder_name: Option<Box<dyn ::druid::Widget<#variant_ty>>>)
    });

    let struct_defaults = input.variants.iter().map(|variant| {
        let builder_name = variant.resolve_builder_name();
        quote!(#builder_name: None)
    });

    let builder_fns = input.variants.iter().map(|variant| {
        let builder_name = variant.resolve_builder_name();
        let variant_ty = &variant.field.ty;
        quote! {
            fn #builder_name(mut self, widget: impl ::druid::Widget<#variant_ty> + 'static) -> Self {
                self.#builder_name = Some(Box::new(widget));
                self
            }
        }
    });

    let widget_added_checks = input.variants.iter().map(|variant| {
        let builder_name = variant.resolve_builder_name();
        quote! {
            if self.default_.is_none() && self.#builder_name.is_none() {
                ::log::warn!("{}::{} variant of {:?} has not been set.", stringify!(#matcher_name), stringify!(#builder_name), ctx.widget_id());
            }
        }
    });

    let event_match = input.variants.iter().map(|variant| {
        let builder_name = variant.resolve_builder_name();
        let variant_name = &variant.name;
        quote! {
            #enum_name::#variant_name(inner) => match &mut self.#builder_name {
                Some(widget) => widget.event(ctx, event, inner, env),
                None => (),
            }
        }
    });

    let lifecycle_match = input.variants.iter().map(|variant| {
        let builder_name = variant.resolve_builder_name();
        let variant_name = &variant.name;
        quote! {
            #enum_name::#variant_name(inner) => match &mut self.#builder_name {
                Some(widget) => widget.lifecycle(ctx, event, inner, env),
                None => (),
            }
        }
    });

    let update_match = input.variants.iter().map(|variant| {
        let builder_name = variant.resolve_builder_name();
        let variant_name = &variant.name;
        quote! {
            (#enum_name::#variant_name(old_inner), #enum_name::#variant_name(inner)) => {
                match &mut self.#builder_name {
                    Some(widget) => widget.update(ctx, old_inner, inner, env),
                    None => (),
                }
            }
        }
    });

    let layout_match = input.variants.iter().map(|variant| {
        let builder_name = variant.resolve_builder_name();
        let variant_name = &variant.name;
        quote! {
            #enum_name::#variant_name(inner) => match &mut self.#builder_name {
                Some(widget) => widget.layout(ctx, bc, inner, env),
                None => bc.min(),
            }
        }
    });

    let paint_match = input.variants.iter().map(|variant| {
        let builder_name = variant.resolve_builder_name();
        let variant_name = &variant.name;
        quote! {
            #enum_name::#variant_name(inner) => match &mut self.#builder_name {
                Some(widget) => widget.paint(ctx, inner, env),
                None => (),
            }
        }
    });

    let output = quote! {
        impl #enum_name {
            pub fn matcher() -> #matcher_name {
                #matcher_name::new()
            }
        }

        struct #matcher_name {
            #(#struct_fields,)*
            default_: Option<Box<dyn ::druid::Widget<#enum_name>>>,
            discriminant_: Option<::std::mem::Discriminant<#enum_name>>,
        }

        impl #matcher_name {
            fn new() -> Self {
                Self {
                    #(#struct_defaults,)*
                    default_: None,
                    discriminant_: None,
                }
            }
            fn default(mut self, widget: impl ::druid::Widget<#enum_name> + 'static) -> Self {
                self.default_ = Some(Box::new(widget));
                self
            }
            fn default_empty(mut self) -> Self {
                self.default_ = Some(Box::new(::druid::widget::SizedBox::empty()));
                self
            }
            #(#builder_fns)*
        }

        impl ::druid::Widget<#enum_name> for #matcher_name {
            fn event(
                &mut self,
                ctx: &mut ::druid::EventCtx,
                event: &::druid::Event,
                data: &mut #enum_name,
                env: &::druid::Env
            ) {
                if self.discriminant_ == Some(::std::mem::discriminant(data)) {
                    match data {
                        #(#event_match)*
                    }
                }
            }
            fn lifecycle(
                &mut self,
                ctx: &mut ::druid::LifeCycleCtx,
                event: &::druid::LifeCycle,
                data: &#enum_name,
                env: &::druid::Env
            ) {
                if let ::druid::LifeCycle::WidgetAdded = event {
                    self.discriminant_ = Some(::std::mem::discriminant(data));
                    #(#widget_added_checks)*
                }
                match data {
                    #(#lifecycle_match)*
                }
            }
            fn update(&mut self,
                ctx: &mut ::druid::UpdateCtx,
                old_data: &#enum_name,
                data: &#enum_name,
                env: &::druid::Env
            ) {
                match (old_data, data) {
                    #(#update_match)*
                    _ => {
                        ctx.children_changed();
                        self.discriminant_ = Some(::std::mem::discriminant(data));
                    }
                }
            }
            fn layout(
                &mut self,
                ctx: &mut ::druid::LayoutCtx,
                bc: &::druid::BoxConstraints,
                data: &#enum_name,
                env: &::druid::Env
            ) -> ::druid::Size {
                match data {
                    #(#layout_match)*
                }
            }
            fn paint(&mut self, ctx: &mut ::druid::PaintCtx, data: &#enum_name, env: &::druid::Env) {
                match data {
                    #(#paint_match)*
                }
            }
        }
    };
    output.into()
}
