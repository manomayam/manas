use manas_server::recipe::{
    impl_::single_pod_noauth::{setup::impl_::FsNoAuthRecipeSetup, SinglePodNoAuthRecipe},
    RecipeExt,
};

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    SinglePodNoAuthRecipe::<FsNoAuthRecipeSetup>::default()
        .main()
        .await
}
