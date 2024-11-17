use async_nats::Client;

pub struct State {
    pub nc: Client
}

impl State {
    pub fn new(nc: Client) -> State {
        State {
            nc
        }
    }
}