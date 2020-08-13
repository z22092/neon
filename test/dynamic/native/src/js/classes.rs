use neon::prelude::*;

pub struct User {
  id: i32,
  first_name: String,
  last_name: String,
  email: String,
}

type Unit = ();

declare_types! {
  pub class JsPanickyAllocator for Unit {
    init(_) {
      panic!("allocator panicking")
    }
  }

  pub class JsPanickyConstructor for Unit {
    init(_) {
      Ok(())
    }

    call(_) {
      panic!("constructor call panicking")
    }

    constructor(_) {
      panic!("constructor panicking")
    }
  }

  pub class JsUser for User {
    init(mut cx) {
      let id = cx.argument::<JsNumber>(0)?;
      let first_name: Handle<JsString> = cx.argument::<JsString>(1)?;
      let last_name: Handle<JsString> = cx.argument::<JsString>(2)?;
      let email: Handle<JsString> = cx.argument::<JsString>(3)?;

      Ok(User {
        id: id.value() as i32,
        first_name: first_name.value(),
        last_name: last_name.value(),
        email: email.value(),
      })
    }

    method get(mut cx) {
      let attr: String = cx.argument::<JsString>(0)?.value();

      let this = cx.this();

      match &attr[..] {
        "id" => {
          let id = {
            let guard = cx.lock();
            let user = this.borrow(&guard);
            user.id
          };
          Ok(cx.number(id).upcast())
        },
        "first_name" => {
          let first_name = {
            let guard = cx.lock();
            let user = this.borrow(&guard);
            user.first_name.clone()
          };
          Ok(cx.string(&first_name).upcast())
        },
        "last_name" => {
          let last_name = {
            let guard = cx.lock();
            let user = this.borrow(&guard);
            user.last_name.clone()
          };
          Ok(cx.string(&last_name).upcast())
        },
        "email" => {
          let email = {
            let guard = cx.lock();
            let user = this.borrow(&guard);
            user.email.clone()
          };
          Ok(cx.string(&email).upcast())
        },
        _ => cx.throw_type_error("property does not exist")
      }
    }

    method panic(_) {
      panic!("User.prototype.panic")
    }

    method alias_self(mut cx) {
      let mut this_mut = cx.this();
      let this_ref = cx.this();
      let first_name = cx.argument::<JsString>(0)?.value();
      let (a, b) = {
        let guard = cx.lock();
        let mut self_mut = this_mut.borrow_mut(&guard);

        // Should panic
        let self_ref = this_ref.borrow(&guard);

        self_mut.first_name = first_name;

        (self_mut.first_name.to_string(), self_ref.first_name.to_string())
      };

      let res = cx.empty_array();
      let a = cx.string(a);
      let b = cx.string(b);

      res.set(&mut cx, 0, a)?;
      res.set(&mut cx, 1, b)?;

      Ok(res.upcast())
    }

    method alias_guard(mut cx) {
      let mut this_mut = cx.this();
      let this_ref = cx.this();
      let first_name = cx.argument::<JsString>(0)?.value();
      let (a, b) = {
        let guard_a = cx.lock();
        let guard_b = cx.lock();
        let mut self_mut = this_mut.borrow_mut(&guard_a);

        // Should panic
        let self_ref = this_ref.borrow(&guard_b);

        self_mut.first_name = first_name;

        (self_mut.first_name.to_string(), self_ref.first_name.to_string())
      };

      let res = cx.empty_array();
      let a = cx.string(a);
      let b = cx.string(b);

      res.set(&mut cx, 0, a)?;
      res.set(&mut cx, 1, b)?;

      Ok(res.upcast())
    }
  }
}
