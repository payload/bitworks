use proc_macro2::TokenStream;
use quote::quote;

pub fn expand_struct(derive_input: &syn::DeriveInput, data: &syn::DataStruct) -> TokenStream {
    let name = derive_input.ident.to_string();

    let fields = data.fields.iter().filter_map(|f| {
        if let Some(ref _ident) = f.ident {
            Some(quote! {
                ui.label(_ident);
                changed |= self.#f.ident.ui(ui, Default::default(), &context.with_id(i));
                ui.end_row();
            })
        } else {
            None
        }
    });

    quote! {
        #[allow(clippy::all)]
        impl bevy_inspector_egui::Inspectable for #name {
            type Attributes = ();

            fn ui(&mut self, ui: &mut bevy_inspector_egui::egui::Ui, options: Self::Attributes, context: &bevy_inspector_egui::Context) -> bool {
                use bevy_inspector_egui::egui;

                let mut changed = false;
                ui.vertical_centered(|ui| {
                    let grid = egui::Grid::new(context.id());
                    grid.show(ui, |ui| {
                        #(#fields)*
                    });
                });
                changed
            }

            fn setup(app: &mut bevy::prelude::AppBuilder) {
                // #(#field_setup)*
            }
        }
    }
}
