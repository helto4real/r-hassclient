use r_hassappruntime::appruntime::APP_RUNTIME;

#[tokio::main]
async fn main() {
    APP_RUNTIME.print();
}
