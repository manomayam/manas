use manas_server::recipe::{
    impl_::single_pod::{setup::impl_::FsWacRecipeSetup, SinglePodRecipe},
    RecipeExt,
};

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    SinglePodRecipe::<FsWacRecipeSetup>::default().main().await
}
