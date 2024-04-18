#[macro_use]
extern crate serde;
use candid::{Decode, Encode};
use ic_cdk::api::time;
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{BoundedStorable, Cell, DefaultMemoryImpl, StableBTreeMap, Storable};
use regex::Regex;
use std::{borrow::Cow, cell::RefCell, error::Error};

type Memory = VirtualMemory<DefaultMemoryImpl>;
type IdCell = Cell<u64, Memory>;

// Custom error type
#[derive(Debug)]
enum CustomError {
    NotFound(String),
    InvalidPayloadData(String),
}

impl Error for CustomError {
    fn description(&self) -> &str {
        match self {
            CustomError::NotFound(msg) => msg,
            CustomError::InvalidPayloadData(msg) => msg,
        }
    }
}

// ... (struct definitions remain the same) ...

impl Storable for User {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

// ... (other impl blocks remain the same) ...

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(
        MemoryManager::init(DefaultMemoryImpl::default())
    );

    static USER_ID_COUNTER: RefCell<IdCell> = RefCell::new(
        IdCell::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))), 0)
            .expect("Cannot create a user ID counter")
    );

    static PRODUCT_ID_COUNTER: RefCell<IdCell> = RefCell::new(
        IdCell::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1))), 0)
            .expect("Cannot create a product ID counter")
    );

    static REVIEW_ID_COUNTER: RefCell<IdCell> = RefCell::new(
        IdCell::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(2))), 0)
            .expect("Cannot create a review ID counter")
    );

    static USER_STORAGE: RefCell<StableBTreeMap<u64, User, Memory>> =
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(3)))
    ));

    static PRODUCT_STORAGE: RefCell<StableBTreeMap<u64, ProductReviewz, Memory>> =
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(4)))
    ));

    static REVIEW_STORAGE: RefCell<StableBTreeMap<u64, Review, Memory>> =
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(5)))
    ));
}

// ... (payload struct definitions remain the same) ...

fn attribute_unique_validation(data: &str, attribute: UniqueAttribue) -> bool {
    let is_unique: bool = !USER_STORAGE.with(|s| {
        s.borrow().iter().any(|(_, user_data)| {
            match attribute {
                UniqueAttribue::Email => user_data.email == data,
                UniqueAttribue::Username => user_data.username == data,
            }
        })
    });

    is_unique
}

// ... (validation functions remain the same) ...

#[ic_cdk::query]
fn view_all_user() -> Option<Vec<User>> {
    USER_STORAGE.with(|s| {
        Some(s.borrow().iter().map(|(_, user_data)| user_data).cloned().collect())
    })
}

#[ic_cdk::query]
fn view_all_product() -> Option<Vec<ProductReviewz>> {
    PRODUCT_STORAGE.with(|s| {
        Some(s.borrow().iter().map(|(_, product_data)| product_data).cloned().collect())
    })
}

#[ic_cdk::query]
fn view_all_review() -> Option<Vec<Review>> {
    REVIEW_STORAGE.with(|s| {
        Some(s.borrow().iter().map(|(_, review_data)| review_data).cloned().collect())
    })
}

#[ic_cdk::update]
fn create_user(data: CreateUserPayload) -> Result<Option<User>, CustomError> {
    let user_data_valid = create_user_validation(&data);

    if !user_data_valid {
        return Err(CustomError::InvalidPayloadData {
            msg: "Invalid data, make sure the email is in valid format, email and username must be unique".to_string(),
        });
    }

    let id = USER_ID_COUNTER
        .with(|counter| {
            let current_value = *counter.borrow().get();
            counter.borrow_mut().set(current_value + 1)
        })
        .expect("cannot increment id counter");
    let new_user = User {
        user_id: id,
        email: data.email,
        username: data.username,
        role: data.role,
        joined_at: time(),
    };

    let insert_success = store_user(&new_user);

    match insert_success {
        true => Ok(Some(new_user)),
        false => Err(CustomError::NotFound {
            msg: "Error while inserting new user".to_string(),
        }),
    }
}

#[ic_cdk::update]
fn add_product(data: AddProductPayload) -> Result<Option<ProductReviewz>, CustomError> {
    let add_product_data_valid = add_product_validation(&data);

    if !add_product_data_valid {
        return Err(CustomError::InvalidPayloadData {
            msg: "Invalid data, user must be valid and must be a store owner, and link must be in a valid format"
                .to_string(),
        });
    }

    let id = PRODUCT_ID_COUNTER
        .with(|counter| {
            let current_value = *counter.borrow().get();
            counter.borrow_mut().set(current_value + 1)
        })
        .expect("cannot increment id counter");

    let new_product = ProductReviewz {
        product_id: id,
        product_name: data.product_name,
        product_description: data.product_description,
        product_link: data.product_link,
        owner_user_id: data.owner_user_id,
    };

    store_product(&new_product);
    Ok(Some(new_product))
}

#[ic_cdk::update]
fn add_review(data: AddReviewPayload) -> Result<Option<Review>, CustomError> {
    let review_data_valid = review_product_validation(&data);

    if !review_data_valid {
        return Err(CustomError::InvalidPayloadData {
            msg: "Invalid data, user and product id must be valid, user must be a customer, rating must be in range 1-5 inclusive"
                .to_string(),
        });
    }

    let id = REVIEW_ID_COUNTER
        .with(|counter| {
            let current_value = *counter.borrow().get();
            counter.borrow_mut().set(current_value + 1)
        })
        .expect("cannot increment id counter");

    let new_review = Review {
        review_id: id,
        product_id: data.product_id,
        user_id: data.user_id,
        rating: data.rating,
        review_description: data.review_description,
    };

    store_review(&new_review);
    Ok(Some(new_review))
}

#[ic_cdk::update]
fn update_product(data: UpdateProductPayload) -> Result<Option<ProductReviewz>, CustomError> {
    let update_product_data_valid = update_product_validation(&data);

    if !update_product_data_valid {
        return Err(CustomError::InvalidPayloadData {
            msg: "Invalid data, user and product id must be valid, user must be the owner of the product"
                .to_string(),
        });
    }

    match PRODUCT_STORAGE.with(|service| service.borrow_mut().get_mut(&data.product_id)) {
        Some(product) => {
            let updated_product = ProductReviewz {
                product_id: data.product_id,
                product_name: data.product_name,
                product_description: data.product_description,
                product_link: data.product_link,
                owner_user_id: product.owner_user_id,
            };
            store_product(&updated_product);
            Ok(Some(updated_product))
        }
        None => Err(CustomError::NotFound {
            msg: format!(
                "Couldn't update a product with id={}. Product not found",
                &data.product_id
            ),
        }),
    }
}

#[ic_cdk::update]
fn delete_product(data: DeleteProductPayload) -> Result<Option<ProductReviewz>, CustomError> {
    let delete_payload_valid = delete_product_validation(&data);

    if !delete_payload_valid {
        return Err(CustomError::InvalidPayloadData {
            msg: "Invalid data, user and product id must be valid, user must be the owner of the product"
                .to_string(),
        });
    }

    let reviews = get_reviews_by_product_id(&data.product_id);

    REVIEW_STORAGE.with(|service| {
        for review in reviews {
            service.borrow_mut().remove(&review.review_id);
        }
    });

    match PRODUCT_STORAGE.with(|service| service.borrow_mut().remove(&data.product_id)) {
        Some(product) => Ok(Some(product)),
        None => Err(CustomError::NotFound {
            msg: format!(
                "Couldn't delete a product with id={}. Product not found.",
                &data.product_id
            ),
        }),
    }
}

// ... (validation functions remain the same) ...

fn get_reviews_by_product_id(id: &u64) -> Vec<Review> {
    REVIEW_STORAGE.with(|service| {
        service
            .borrow()
            .iter()
            .filter_map(|(_, review)| {
                if review.product_id == *id {
                    Some(review.clone())
                } else {
                    None
                }
            })
            .collect()
    })
}

fn store_user(data: &User) -> bool {
    let result = USER_STORAGE.with(|service| service.borrow_mut().insert(data.user_id, data.clone()));
    match result {
        Some(_) => false,
        None => true,
    }
}

fn store_product(data: &ProductReviewz) {
    PRODUCT_STORAGE
        .with(|service| service.borrow_mut().insert(data.product_id, data.clone()));
}

fn store_review(data: &Review) {
    REVIEW_STORAGE
        .with(|service| service.borrow_mut().insert(data.review_id, data.clone()));
}

fn get_user_by_id(user_id: &u64) -> Option<User> {
    USER_STORAGE.with(|s| s.borrow().get(user_id).cloned())
}

fn get_product_by_id(product_id: &u64) -> Option<ProductReviewz> {
    PRODUCT_STORAGE.with(|s| s.borrow().get(product_id).cloned())
}

#[ic_cdk::update]
fn clear_all_user() {
    USER_ID_COUNTER.with(|counter| {
        let count = *counter.borrow().get();
        for i in 0..count {
            USER_STORAGE.with(|service| service.borrow_mut().remove(&i));
        }
    });
}

#[ic_cdk::update]
fn clear_all_product() {
    PRODUCT_ID_COUNTER.with(|counter| {
        let count = *counter.borrow().get();
        for i in 0..count {
            PRODUCT_STORAGE.with(|service| service.borrow_mut().remove(&i));
        }
    });
}

ic_cdk::export_candid!();
