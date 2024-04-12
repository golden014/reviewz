#[macro_use]
extern crate serde;
use candid::{Decode, Encode}; //serialization format used in ICP
use ic_cdk::api::time; //core crate for rust Canister Development kit, provide core method supaya program rust bisa interact
//dengan Internet Computer Blockchain system API
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory}; //provide data structures
use ic_stable_structures::{BoundedStorable, Cell, DefaultMemoryImpl, StableBTreeMap, Storable};
use regex::Regex;
use std::{borrow::Cow, cell::RefCell};

// #[ic_cdk::query]
// fn greet(name: String) -> String {
//     format!("Hello, {}!", name)
// }

type Memory = VirtualMemory<DefaultMemoryImpl>;
type IdCell = Cell<u64, Memory>; //cell responsible for holding the current ID of the message. 
//We'll utilize this to generate unique IDs for each message.


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

// a trait that must be implemented for a struct that is stored in a stable struct - User
impl Storable for User {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

// A trait indicating that a `Storable` element is bounded in size - User
impl BoundedStorable for User {
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
    // static PRODUCT_STORAGE: RefCell<StableBTreeMap<u64, User, Memory>> =
    //     RefCell::new(StableBTreeMap::init(
    //         MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(3)))
    // ));

    // static REVIEW_STORAGE: RefCell<StableBTreeMap<u64, User, Memory>> =
    //     RefCell::new(StableBTreeMap::init(
    //         MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(3)))
    // ));
    
}

//The payloads struct defines the structure for the data that will be used when creating or updating something within our canister.
#[derive(candid::CandidType, Serialize, Deserialize, Default)]
struct CreateUserPayload {
    email: String,
    username: String,
    role: String
}

//enum for errors
#[derive(candid::CandidType, Deserialize, Serialize)]
enum Error {
    NotFound { msg: String },
    InvalidUserData {msg: String}
}

// //retrieves a message from our canister's storage.
// #[ic_cdk::query]
// fn get_message(id: u64) -> Result<Message, Error> {
//     match _get_message(&id) {
//         Some(message) => Ok(message),
//         None => Err(Error::NotFound {
//             msg: format!("a message with id={} not found", id),
//         }),
//     }
// }

// //helper utk dipake di get_message
// fn _get_message(id: &u64) -> Option<Message> {
//     STORAGE.with(|s| s.borrow().get(id))
// }
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

//takes a message of type MessagePayload as input and returns an Option<Message>. It generates a unique id for the message, creates a new Message struct, and adds it to the canister's storage
// #[ic_cdk::update]
// fn add_message(message: MessagePayload) -> Option<Message> {
//     let id = ID_COUNTER
//         .with(|counter| {
//             let current_value = *counter.borrow().get();
//             counter.borrow_mut().set(current_value + 1)
//         })
//         .expect("cannot increment id counter");
//     let message = Message {
//         id,
//         title: message.title,
//         body: message.body,
//         attachment_url: message.attachment_url,
//         created_at: time(),
//         updated_at: None,
//     };
//     do_insert(&message);
//     Some(message)
// }

#[ic_cdk::query]
fn view_all_user() -> Option<Vec<User>> {
    USER_STORAGE.with(|s| {
        //iterating through the USER_STORAGE's key-value pair, take all the value (user-data) and return as a vector
        Some(s.borrow().iter().map(|(_user_id, user_data)| (user_data.clone())).collect())
    })
}

#[ic_cdk::update]
fn create_user(data: CreateUserPayload) -> Result<Option<User>, Error> {
    let user_data_valid = create_user_validation(&data);

    if user_data_valid == false {
        return Result::Err(Error::InvalidUserData { msg: "Invalid data, make sure the email is in valid format, email and username must be unique".to_string() })
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
    do_insert_user(&new_user);
    return Result::Ok(Some(new_user)) 
}

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

// helper method to perform insert, insert the new user data
fn do_insert_user(data: &User) {
    USER_STORAGE.with(|service| service.borrow_mut().insert(data.user_id, data.clone()));
}

// #[ic_cdk::update]
// fn update_message(id: u64, payload: MessagePayload) -> Result<Message, Error> {
//     match STORAGE.with(|service| service.borrow().get(&id)) {
//         Some(mut message) => {
//             message.attachment_url = payload.attachment_url;
//             message.body = payload.body;
//             message.title = payload.title;
//             message.updated_at = Some(time());
//             do_insert(&message);
//             Ok(message)
//         }
//         None => Err(Error::NotFound {
//             msg: format!(
//                 "couldn't update a message with id={}. message not found",
//                 id
//             ),
//         }),
//     }
// }

// #[ic_cdk::update]
// fn delete_message(id: u64) -> Result<Message, Error> {
//     match STORAGE.with(|service| service.borrow_mut().remove(&id)) {
//         Some(message) => Ok(message),
//         None => Err(Error::NotFound {
//             msg: format!(
//                 "couldn't delete a message with id={}. message not found.",
//                 id
//             ),
//         }),
//     }
// }

// need this to generate candid
ic_cdk::export_candid!();
