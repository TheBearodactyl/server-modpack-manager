# How to use

1. Download the utility from releases
2. Double click it (you may need to tell Windows to run anyway)
3. Choose your mod manager
4. And boom, you're done :3

# Usage with custom server modpacks

1. Create zipped versions of each mod managers version of your modpack (TODO: More detailed instructions)
2. Upload them to releases as `updated-pack-curseforge.zip` and `updated-pack-prism.zip` (Modrinth is currently non-functional)
3. Set the repo in `.env` to the repo where you'll be uploading releases to as follows:
```env
REPO_USER="yourgithubusername"
REPO_NAME="thenameoftheghrepo"
```
4. Compile via `cargo build --release`
5. Then distribute the utility to the people using it :3
