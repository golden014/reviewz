#[macro_use]
extern crate serde;
use candid::{Decode, Encode}; //serialization format used in ICP
use ic_cdk::api::time; //core crate for rust Canister Development kit, provide core method supaya program rust bisa interact
//dengan Internet Computer Blockchain system API
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory}; //provide data structures
use ic_stable_structures::{BoundedStorable, Cell, DefaultMemoryImpl, StableBTreeMap, Storable};
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
struct Message {
    id: u64,
    title: String,
    body: String,
    attachment_url: String,
    created_at: u64,
    updated_at: Option<u64>,
}

// a trait that must be implemented for a struct that is stored in a stable struct
impl Storable for Message {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

// another trait that must be implemented for a struct that is stored in a stable struct
// A trait indicating that a `Storable` element is bounded in size.
impl BoundedStorable for Message {
    const MAX_SIZE: u32 = 1024;
    const IS_FIXED_SIZE: bool = false;
}
//thread-local variables that will hold our canister's state. Thread-local variables are variables that are local to the current thread (sequence of instructions). They are useful when you need to share data between multiple threads.

//RefCell to manage our canister's state, allowing us to access it from anywhere in our code. Tapi compiler gabisa guarantee our code adheres to the borrowing rules
thread_local! {
    //This thread-local variable holds our canister's virtual memory, enabling us to access the memory manager from any part of our code.
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(
        MemoryManager::init(DefaultMemoryImpl::default())
    );

    //It holds our canister's ID counter, allowing us to access it from anywhere in our code.
    static ID_COUNTER: RefCell<IdCell> = RefCell::new(
        IdCell::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))), 0)
            .expect("Cannot create a counter")
    );

    //This variable holds our canister's storage, enabling access from anywhere in our code.
    static STORAGE: RefCell<StableBTreeMap<u64, Message, Memory>> =
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1)))
    ));
}

//The MessagePayload struct defines the structure for the data that will be used when creating or updating messages within our canister.
#[derive(candid::CandidType, Serialize, Deserialize, Default)]
struct MessagePayload {
    title: String,
    body: String,
    attachment_url: String,
}

//enum utk error
#[derive(candid::CandidType, Deserialize, Serialize)]
enum Error {
    NotFound { msg: String },
}

//retrieves a message from our canister's storage.
#[ic_cdk::query]
fn get_message(id: u64) -> Result<Message, Error> {
    match _get_message(&id) {
        Some(message) => Ok(message),
        None => Err(Error::NotFound {
            msg: format!("a message with id={} not found", id),
        }),
    }
}

//helper utk dipake di get_message
fn _get_message(id: &u64) -> Option<Message> {
    STORAGE.with(|s| s.borrow().get(id))
}

//takes a message of type MessagePayload as input and returns an Option<Message>. It generates a unique id for the message, creates a new Message struct, and adds it to the canister's storage
#[ic_cdk::update]
fn add_message(message: MessagePayload) -> Option<Message> {
    let id = ID_COUNTER
        .with(|counter| {
            let current_value = *counter.borrow().get();
            counter.borrow_mut().set(current_value + 1)
        })
        .expect("cannot increment id counter");
    let message = Message {
        id,
        title: message.title,
        body: message.body,
        attachment_url: message.attachment_url,
        created_at: time(),
        updated_at: None,
    };
    do_insert(&message);
    Some(message)
}

// helper method to perform insert.
fn do_insert(message: &Message) {
    STORAGE.with(|service| service.borrow_mut().insert(message.id, message.clone()));
}

#[ic_cdk::update]
fn update_message(id: u64, payload: MessagePayload) -> Result<Message, Error> {
    match STORAGE.with(|service| service.borrow().get(&id)) {
        Some(mut message) => {
            message.attachment_url = payload.attachment_url;
            message.body = payload.body;
            message.title = payload.title;
            message.updated_at = Some(time());
            do_insert(&message);
            Ok(message)
        }
        None => Err(Error::NotFound {
            msg: format!(
                "couldn't update a message with id={}. message not found",
                id
            ),
        }),
    }
}

#[ic_cdk::update]
fn delete_message(id: u64) -> Result<Message, Error> {
    match STORAGE.with(|service| service.borrow_mut().remove(&id)) {
        Some(message) => Ok(message),
        None => Err(Error::NotFound {
            msg: format!(
                "couldn't delete a message with id={}. message not found.",
                id
            ),
        }),
    }
}

// need this to generate candid
ic_cdk::export_candid!();
