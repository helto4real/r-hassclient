use crate::{HaEventData, HassError, Response};

#[test]
fn state_chage_should_parse() {
    // open file as string
    let payload: Result<Response, crate::HassError> = serde_json::from_str(
        r#"
    {
      "id": 1,
      "type": "event",
      "event": {
        "event_type": "state_changed",
        "data": {
          "entity_id": "input_boolean.test",
          "old_state": {
            "entity_id": "input_boolean.test",
            "state": "off",
            "attributes": {
              "editable": true,
              "friendly_name": "test"
            },
            "last_changed": "2023-08-28T09:08:13.985677+00:00",
            "last_updated": "2023-08-28T09:08:13.985677+00:00",
            "context": {
              "id": "01H8XPD611JJ7Q3WP5VT5FVEWN",
              "parent_id": null,
              "user_id": "f89f13024806490b8d879160843ddf54"
            }
          },
          "new_state": {
            "entity_id": "input_boolean.test",
            "state": "on",
            "attributes": {
              "editable": true,
              "friendly_name": "test"
            },
            "last_changed": "2023-08-28T09:09:05.471838+00:00",
            "last_updated": "2023-08-28T09:09:05.471838+00:00",
            "context": {
              "id": "01H8XPER9ZAGWM4P3WZQ7BPKPR",
              "parent_id": null,
              "user_id": "f89f13024806490b8d879160843ddf54"
            }
          }
        },
        "origin": "LOCAL",
        "time_fired": "2023-08-28T09:09:05.471838+00:00",
        "context": {
          "id": "01H8XPER9ZAGWM4P3WZQ7BPKPR",
          "parent_id": null,
          "user_id": "f89f13024806490b8d879160843ddf54"
        }
      }
    }"#,
    )
    .map_err(|e| HassError::GenericError(e.to_string()));

    match payload {
        Ok(response) => match response {
            Response::Event(event) => {
                let event_data = event.event.get_event_data();
                match event_data {
                    Ok(HaEventData::StateChangedEvent(event)) => {
                        assert_eq!(event.entity_id, "input_boolean.test");
                        let new_state = event.new_state.unwrap();
                        let old_state = event.old_state.unwrap();
                        assert_eq!(new_state.state, "on");
                        assert_eq!(new_state.attributes.unwrap()["friendly_name"], "test");
                        assert_eq!(old_state.state, "off");
                    }
                    _ => {
                        panic!("We should have an event response 2!");
                    }
                }
            }
            x => {
                panic!("We should have an event response 2! {:?}", x);
            }
        },
        Err(err) => {
            println!("Error: {:?}", err);
            panic!("we should had a valid response!1");
        }
    }
}
