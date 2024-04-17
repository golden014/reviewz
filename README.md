# REVIEWZ
Reviewz is a platform where each user can review their favorite product. If they are a store owner, they can add their own product into the system and also manage their product.

Features:

1. create_user(email, username, role)
to create a new user and insert it into the system, email must be in a valid email format, email and username must be unique, and role must be either customer or store_owner

2. add_product(product_name, product_description, product_link, added_by)
validate that the product must be added by user with store_owner role, product_link must be a valid link format


3. delete_product(product_id, user_id)
to delete a product from the system, product_id and user_id must be valid, validate the user that wants to delete it is the one that inserted it. Also delete all the reviews for the specified product.

4. update_product(product_id, product_name, product_description, user_id)
to update a products name/description/link, validate the user must be the one that inserted the product, product_id and user_id must be valid.

5. view_all_products()
retrieve all products

6. add_product_review(product_id, user_id, rating, review_description)
product_id and user_id must be valid, user’s role must be a customer, rating cant exceed 5 (1-5 inclusive)

### Requirements
* rustc 1.64 or higher
```bash
$ curl --proto '=https' --tlsv1.2 https://sh.rustup.rs -sSf | sh
$ source "$HOME/.cargo/env"
```
* rust wasm32-unknown-unknown target
```bash
$ rustup target add wasm32-unknown-unknown
```
* candid-extractor
```bash
$ cargo install candid-extractor
```
* install `dfx`
```bash
$ DFX_VERSION=0.19.0 sh -ci "$(curl -fsSL https://sdk.dfinity.org/install.sh)"
$ echo 'export PATH="$PATH:$HOME/bin"' >> "$HOME/.bashrc"
$ source ~/.bashrc
$ dfx start --background
```

If you want to start working on your project right away, you might want to try the following commands:

```bash
$ cd reviewz/
$ dfx help
$ dfx canister --help
```

## Update dependencies

update the `dependencies` block in `/src/{canister_name}/Cargo.toml`:
```
[dependencies]
candid = "0.9.9"
ic-cdk = "0.11.1"
serde = { version = "1", features = ["derive"] }
serde_json = "1.0"
ic-stable-structures = { git = "https://github.com/lwshang/stable-structures.git", branch = "lwshang/update_cdk"}
```

## did autogenerate

```
{
    "scripts": {
        "generate": "./did.sh && dfx generate",
        "gen-deploy": "./did.sh && dfx generate && dfx deploy -y"
      }
}
```

use commands `npm run generate` to generate candid or `npm run gen-deploy` to generate candid and to deploy a canister.

## Running the project locally

If you want to run this project locally, you can use the following commands:

```bash
# Starts the replica, running in the background
$ dfx start --background

# Deploys your canisters to the replica and generates your candid interface
$ dfx deploy
```
