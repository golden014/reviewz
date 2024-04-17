#[macro_use]
extern crate serde;
use candid::{Decode, Encode};
use ic_cdk::api::time; 
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{BoundedStorable, Cell, DefaultMemoryImpl, StableBTreeMap, Storable};
use regex::Regex;
use std::{borrow::Cow, cell::RefCell};


type Memory = VirtualMemory<DefaultMemoryImpl>;
type IdCell = Cell<u64, Memory>;


//This struct will represent the messages in our message board application, and it 
//includes fields for ID, title, body, attachment URL, creation timestamp, and an optional update timestamp.

//this attribute is telling the Rust compiler to automatically implement the CandidType trait from the candid crate, as well as the Clone, Serialize, Deserialize, and Default traits for the struct or enum that this attribute is applied to. This is a common pattern used in Rust when working with serialization and deserialization libraries to provide automatic implementation of necessary traits for working with those libraries.
#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct User {
    user_id: u64,
    email: String,
    username: String,
    role: String,
    joined_at: u64
}
//the product
#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct ProductReviewz {
    product_id: u64,
    product_name: String,
    product_description: String,
    product_link: String,
    owner_user_id: u64
}
//review
#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct Review {
    review_id: u64,
    product_id: u64,
    user_id: u64,
    rating: u64,
    review_description: String,
}

// a trait that must be implemented for a struct that is stored in a stable struct
impl Storable for User {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}
impl Storable for ProductReviewz {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}
impl Storable for Review {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

// A trait indicating that a `Storable` element is bounded in size
impl BoundedStorable for User {
    const MAX_SIZE: u32 = 1024;
    const IS_FIXED_SIZE: bool = false;
}
impl BoundedStorable for ProductReviewz {
    const MAX_SIZE: u32 = 1024;
    const IS_FIXED_SIZE: bool = false;
}
impl BoundedStorable for Review {
    const MAX_SIZE: u32 = 1024;
    const IS_FIXED_SIZE: bool = false;
}

//thread-local variables that will hold our canister's state. Thread-local variables are variables that are local to the current thread (sequence of instructions). They are useful when you need to share data between multiple threads.
thread_local! {
    //This thread-local variable holds our canister's virtual memory, enabling us to access the memory manager from any part of our code.
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(
        MemoryManager::init(DefaultMemoryImpl::default())
    );

    //It holds our canister's user ID counter, allowing us to access it from anywhere in our code.
    static USER_ID_COUNTER: RefCell<IdCell> = RefCell::new(
        IdCell::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))), 0)
            .expect("Cannot create a user ID counter")
    );

    //for product ID 
    static PRODUCT_ID_COUNTER: RefCell<IdCell> = RefCell::new(
        //use different memory region by specifying different memory ID
        IdCell::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1))), 0)
            .expect("Cannot create a product ID counter")
    );

    //for review ID
    static REVIEW_ID_COUNTER: RefCell<IdCell> = RefCell::new(
        IdCell::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(2))), 0)
            .expect("Cannot create a review ID counter")
    );

    //This variable holds our canister's user storage, enabling access from anywhere in our code.
    static USER_STORAGE: RefCell<StableBTreeMap<u64, User, Memory>> =
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(3)))
    ));

    //lanjut after declaring the structs
    static PRODUCT_STORAGE: RefCell<StableBTreeMap<u64, ProductReviewz, Memory>> =
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(4)))
    ));

    static REVIEW_STORAGE: RefCell<StableBTreeMap<u64, Review, Memory>> =
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(5)))
    ));
    
}

//The payloads struct defines the structure for the data that will be used when creating or updating something within our canister.
#[derive(candid::CandidType, Serialize, Deserialize, Default)]
struct CreateUserPayload {
    email: String,
    username: String,
    role: String
}
#[derive(candid::CandidType, Serialize, Deserialize, Default)]
struct AddProductPayload {
    product_name: String,
    product_description: String,
    product_link: String,
    owner_user_id: u64
}
#[derive(candid::CandidType, Serialize, Deserialize, Default)]
struct AddReviewPayload {
    product_id: u64,
    user_id: u64,
    rating: u64,
    review_description: String
}

#[derive(candid::CandidType, Serialize, Deserialize, Default)]
struct UpdateProductPayload {
    product_id: u64,
    user_id: u64,
    product_name: String,
    product_description: String,
    product_link: String
}

#[derive(candid::CandidType, Serialize, Deserialize, Default)]
struct DeleteProductPayload {
    product_id: u64,
    user_id: u64
}

//enum for errors
#[derive(candid::CandidType, Deserialize, Serialize)]
enum Error {
    NotFound { msg: String },
    InvalidPayloadData {msg: String}
}
//enum for unique attribute
#[derive(candid::CandidType, Deserialize, Serialize)]
enum UniqueAttribue{
    Email,
    Username
}

//to check if the specified attribute is unique or not
fn attribute_unique_validation(data: &String, attribute: UniqueAttribue) -> bool {
    let is_unique: bool = !USER_STORAGE.with(|s| {
        s.borrow().iter().any(|(_, user_data)| {
            match attribute {
                //will check different attribute based on the UniqueAttribute passed
                UniqueAttribue::Email => user_data.email == *data,
                UniqueAttribue::Username => user_data.username == *data
            }
        })
    });

    is_unique
}

//to validate user's role
fn role_validation(data: &String) -> bool {
    data == "Customer" || data == "StoreOwner"
}

fn is_store_owner_validation(data: &String) -> bool {
    data == "StoreOwner"
}

fn is_customer_validation(data: &String) -> bool {
    data == "Customer"
}
//rating must be between 1 and 5 (inclusive)
fn _rating_validation(data: &u64) -> bool {
    *data >= 1 && *data <= 5
}

//view all user
#[ic_cdk::query]
fn view_all_user() -> Option<Vec<User>> {
    USER_STORAGE.with(|s| {
        //iterating through the USER_STORAGE's key-value pair, take all the value (user-data) and return as a vector
        Some(s.borrow().iter().map(|(_user_id, user_data)| (user_data.clone())).collect())
    })
}

//view all product
#[ic_cdk::query]
fn view_all_product() -> Option<Vec<ProductReviewz>> {
    PRODUCT_STORAGE.with(|s| {
        //iterating through the PRODUCT_STORAGE's key-value pair, take all the value (user-data) and return as a vector
        Some(s.borrow().iter().map(|(_product_id, product_data)| (product_data.clone())).collect())
    })
}

//view all review
#[ic_cdk::query]
fn view_all_review() -> Option<Vec<Review>> {
    REVIEW_STORAGE.with(|s| {
        //iterating through the REVIEW_STORAGE's key-value pair, take all the value (user-data) and return as a vector
        Some(s.borrow().iter().map(|(_review_id, review_data)| (review_data.clone())).collect())
    })
}

//create new user
#[ic_cdk::update]
fn create_user(data: CreateUserPayload) -> Result<Option<User>, Error> {
    //validate new user's data
    let user_data_valid = create_user_validation(&data);

    if user_data_valid == false {
        return Result::Err(Error::InvalidPayloadData { msg: "Invalid data, make sure the email is in valid format, email and username must be unique".to_string() })
    }

    //get the new id
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

    //insert new User
    let insert_success = do_insert_user(&new_user);

    match insert_success {
        true => return Result::Ok(Some(new_user)),
        false => return Result::Err(Error::NotFound { msg: "error while inserting new user".to_string() })
    }

}


//add new product
#[ic_cdk::update]
fn add_product(data: AddProductPayload) -> Result<Option<ProductReviewz>, Error> {
    let add_product_data_valid = add_product_validation(&data);

    if add_product_data_valid == false {
        return Result::Err(Error::InvalidPayloadData{ msg: "Invalid data, user must be valid and must be a store owner, and link must be in a valid format".to_string() })
    } 
    //get the new id
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
    print!("{}", &new_product.product_id);
    do_insert_product(&new_product);
    return Result::Ok(Some(new_product)) 
}

#[ic_cdk::update]

fn add_review(data: AddReviewPayload) -> Result<Option<Review>, Error> {
    let review_data_valid = review_product_validation(&data);

    if review_data_valid == false {
        return Result::Err(Error::InvalidPayloadData{ msg: "Invalid data, user and product id must be valid, user must be a customer, rating must be in range 1-5 inclusive".to_string() })
    };

    //get the new id
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
        review_description: data.review_description
    };

    do_insert_review(&new_review);
    return Result::Ok(Some(new_review))

}

//update product
#[ic_cdk::update]
fn update_product(data: UpdateProductPayload) -> Result<Option<ProductReviewz>, Error> {
    let update_product_data_valid = update_product_validation(&data);

    if update_product_data_valid == false {
        return Result::Err(Error::InvalidPayloadData{ msg: "Invalid data, user and product id must be valid, user must be the owner of the prodcut".to_string() })
    }

    match PRODUCT_STORAGE.with(|service| service.borrow().get(&data.product_id)) {
        Some(mut product) => {
            product.product_name = data.product_name;
            product.product_description = data.product_description;
            product.product_link = data.product_link;
            do_insert_product(&product);
            Ok(Some(product))
        }
        None => Err(Error::NotFound {
            msg: format!(
                "couldn't update a product with id={}. message not found",
                &data.product_id
            ),
        }),
    }
}

//delete product
#[ic_cdk::update]
fn delete_product(data: DeleteProductPayload) -> Result<Option<ProductReviewz>, Error> {
    //check payload validation
    let delete_payload_valid = delete_product_validation(&data);


    if delete_payload_valid == false {
        return Result::Err(Error::InvalidPayloadData{ msg: "Invalid data, user and product id must be valid, user must be the owner of the prodcut".to_string() })
    }

    //get all the reviews of the product
    let reviews = get_reviews_by_product_id(&data.product_id);

    //iterate through all the reviews and remove them
    REVIEW_STORAGE.with(|service| {
        for review in reviews {
            service.borrow_mut().remove(&review.review_id);
        }
    });

    match PRODUCT_STORAGE.with(|service| service.borrow_mut().remove(&data.product_id)) {
        Some(message) => Ok(Some(message)),
        None => Err(Error::NotFound {
            msg: format!(
                "couldn't delete a message with id={}. message not found.",
                &data.product_id
            ),
        }),
    }

}

//validation when creating a new user
fn create_user_validation(data: &CreateUserPayload) -> bool {
    //email format validation using regex
    let email_format = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();

    //check if the email matches the regex
    let email_format_valid = email_format.is_match(&data.email);

    //role valid
    let role_valid = role_validation(&data.role);
    //check if email and username is unique
    let email_unique = attribute_unique_validation(&data.email, UniqueAttribue::Email); 
    let username_unique = attribute_unique_validation(&data.username, UniqueAttribue::Username);

    return email_format_valid && role_valid && email_unique && username_unique;
}


//add product validation
fn add_product_validation(data: &AddProductPayload) -> bool {
    //get the user
    let user = _get_user_by_id(&data.owner_user_id);

    //if user exist, proceed to check is store owner validation, else return false
    let user_role_valid = match user {
        Some(ref _user) => is_store_owner_validation(&user.unwrap().role),
        None => false
    };

    let valid_link_format = Regex::new(r"^https?://(?:www\.)?[a-zA-Z0-9\-]+\.[a-zA-Z]{2,}(?:/[\w\-./?%&=]*)?$").unwrap();
    //check if the link match the regex
    let link_valid = valid_link_format.is_match(&data.product_link);
    
    return user_role_valid && link_valid;
    
}

fn review_product_validation(data: &AddReviewPayload) -> bool {
    let product = _get_product_by_id(&data.product_id);
    let user = _get_user_by_id(&data.user_id);

    //check if product exist
    let product_valid = match product {
        Some(_product_reviewz) => true,
        None => false
    };

    //check if user exist and the role is customer
    let user_valid = match user {
        Some(ref _user) => is_customer_validation(&user.unwrap().role),
        None => false
    };

    let rating_valid = _rating_validation(&data.rating);

    product_valid && user_valid && rating_valid
}

fn update_product_validation(data: &UpdateProductPayload) -> bool {
    let product = _get_product_by_id(&data.product_id);
    let user = _get_user_by_id(&data.user_id);

    let mut user_id_payload: Option<&u64> = None;
    let mut user_id_product: Option<&u64> = None;

    //check if user exist
    match user {
        Some(ref _user) => user_id_payload = Some(&data.user_id),
        None => {}
    };

    //check if product exist
    match product {
        Some(_product_reviewz) => {
            user_id_product = Some(&_product_reviewz.owner_user_id);

            //if the user specified in the payload is the owner of the product return true
            if user_id_payload.is_some() && user_id_product.is_some() {
                return user_id_payload == user_id_product
            } else {
                return false;
            }
        },
        None => {return false}
    }; 
}

//delete product validation, this can be improved because it does the same thing as the update product validation, just different payload
fn delete_product_validation(data: &DeleteProductPayload) -> bool {
    let product = _get_product_by_id(&data.product_id);
    let user = _get_user_by_id(&data.user_id);

    let mut user_id_payload: Option<&u64> = None;
    let mut user_id_product: Option<&u64> = None;

     //check if user exist
     match user {
        Some(ref _user) => user_id_payload = Some(&data.user_id),
        None => {}
    };

    match product {
        Some(_product_reviewz) => {
            user_id_product = Some(&_product_reviewz.owner_user_id);

            //if the user specified in the payload is the owner of the product return true
            if user_id_payload.is_some() && user_id_product.is_some() {
                return user_id_payload == user_id_product
            } else {
                return false;
            }
        },
        None => {return false}
    }; 
}

//get all the reviews for a specifed product
fn get_reviews_by_product_id(id: &u64) -> Vec<Review> {
    REVIEW_STORAGE.with(|service| {
        service
        .borrow()
        .iter()
        //take every review with the same product id as specified
        .filter_map(|(_, review)| {
            if review.product_id == *id {
                Some(review.clone())
            } else {
                None
            }
        }).collect()
    })  
}

// helper method to perform insert, insert the new user data
fn do_insert_user(data: &User) -> bool {
    let a = USER_STORAGE.with(|service| service.borrow_mut().insert(data.user_id, data.clone()));
    match  a {
        Some(_user) => return false,
        None => return true,
    }
}

fn do_insert_product(data: &ProductReviewz) {
    PRODUCT_STORAGE.with(|service| service.borrow_mut().insert(data.product_id, data.clone()));
}

fn do_insert_review(data: &Review) {
    REVIEW_STORAGE.with(|service| service.borrow_mut().insert(data.review_id, data.clone()));
}

//get user by the specified id
fn _get_user_by_id(user_id: &u64) -> Option<User> {
    USER_STORAGE.with(|s| s.borrow().get(user_id))
}

//get product by the specified id
fn _get_product_by_id(product_id: &u64) -> Option<ProductReviewz> {
    PRODUCT_STORAGE.with(|s| s.borrow().get(product_id))
}

//clear all user - for debug purposes
#[ic_cdk::update]
fn clear_all_user() {
    USER_ID_COUNTER
        .with(|counter| {
            let count = *counter.borrow().get();
        
        //iterate through all the items and remove them
        for i in 0..count{
            USER_STORAGE.with(|service| service.borrow_mut().remove(&i));
        }
    });
}

//clear all user - for debug purposes
#[ic_cdk::update]
fn clear_all_product() {
    PRODUCT_ID_COUNTER
        .with(|counter| {
            let count = *counter.borrow().get();
        
        //iterate through all the items and remove them
        for i in 0..count{
            PRODUCT_STORAGE.with(|service| service.borrow_mut().remove(&i));
        }
    });
}

// need this to generate candid
ic_cdk::export_candid!();
