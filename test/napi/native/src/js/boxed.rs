use std::sync::Arc;
use std::cell::RefCell;

use neon::prelude::*;
use neon::sync::{EventQueue, Persistent};

pub struct Person {
    name: String,
}

impl Person {
    fn new(name: impl ToString) -> Self {
        Self { name: name.to_string() }
    }

    fn greet(&self) -> String {
        format!("Hello, {}!", self.name)
    }

    fn set_name(&mut self, name: impl ToString) {
        self.name = name.to_string();
    }
}

pub fn person_new(mut cx: FunctionContext) -> JsResult<JsBox<Person>> {
    let name = cx.argument::<JsString>(0)?.value(&mut cx);
    let person = Person::new(name);

    Ok(cx.boxed(person))
}

pub fn person_greet(mut cx: FunctionContext) -> JsResult<JsString> {
    let person = cx.argument::<JsBox<Person>>(0)?;
    let greeting = cx.string(person.greet());

    Ok(greeting)
}

pub fn ref_person_new(mut cx: FunctionContext) -> JsResult<JsValue> {
    let name = cx.argument::<JsString>(0)?.value(&mut cx);
    let person = RefCell::new(Person::new(name));

    Ok(cx.boxed(person).upcast())
}

pub fn ref_person_greet(mut cx: FunctionContext) -> JsResult<JsString> {
    let person = cx.argument::<JsBox<RefCell<Person>>>(0)?;
    let greeting = cx.string(person.borrow().greet());

    Ok(greeting)
}

pub fn ref_person_set_name(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let person = cx.argument::<JsBox<RefCell<Person>>>(0)?;
    let name = cx.argument::<JsString>(1)?.value(&mut cx);

    person.borrow_mut().set_name(name);

    Ok(cx.undefined())
}

pub fn ref_person_fail(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let person = cx.argument::<JsBox<RefCell<Person>>>(0)?;
    let _borrow = person.borrow();
    let _borrow_mut = person.borrow_mut();

    Ok(cx.undefined())
}

pub fn external_unit(mut cx: FunctionContext) -> JsResult<JsBox<()>> {
    Ok(cx.boxed(()))
}

pub fn thread_callback(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let callback = cx.argument::<JsFunction>(0)?;
    let callback = Persistent::new(&mut cx, callback);
    let queue = EventQueue::new(&mut cx);

    std::thread::spawn(move || queue.send(move |mut cx| {
        let callback = callback.deref(&mut cx);
        let this = cx.undefined();
        let args = Vec::<Handle<JsValue>>::new();

        callback.call(&mut cx, this, args)
    }));

    Ok(cx.undefined())
}

pub fn multi_threaded_callback(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let n = cx.argument::<JsNumber>(0)?.value(&mut cx);
    let callback = cx.argument::<JsFunction>(1)?;
    let callback = Persistent::new(&mut cx, callback);
    let queue = Arc::new(EventQueue::new(&mut cx));

    for i in 0..(n as usize) {
        let callback = callback.clone(&mut cx);
        let queue = queue.clone();

        std::thread::spawn(move || queue.send(move |mut cx| {
            let callback = callback.deref(&mut cx);
            let this = cx.undefined();
            let args = vec![cx.number(i as f64)];
    
            callback.call(&mut cx, this, args)
        }));
    }

    callback.drop(&mut cx);

    Ok(cx.undefined())
}
