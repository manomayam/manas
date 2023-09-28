use manas_server::recipe::{
    impl_::single_pod::{setup::impl_::S3WacRecipeSetup, SinglePodRecipe},
    RecipeExt,
};

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    SinglePodRecipe::<S3WacRecipeSetup>::default().main().await
}
