use std::fs::File;
use std::io::{BufWriter, Write};

use aws_config::meta::region::RegionProviderChain;
use aws_sdk_sqs::Client;
use aws_sdk_sqs::types::Message;
use dialoguer::MultiSelect;

#[tokio::main]
async fn main() {
    let region_provider = RegionProviderChain::default_provider().or_else("us-east-1");
    let config = aws_config::from_env().region(region_provider).load().await;
    let client = Client::new(&config);
    let response = client.list_queues().send().await;

    match response {
        Ok(output) => {
            // Extract the list of queue URLs from the response
            if let Some(queue_urls) = output.queue_urls {
                let selection = MultiSelect::new()
                    .items(&queue_urls)
                    .interact()
                    .unwrap();
                println!("You chose:");

                for i in selection {
                    println!("{}", queue_urls[i]);
                    // Download all messages and output them to a ND JSON file
                    let messages = client.receive_message()
                        .queue_url(queue_urls[i].clone())
                        .send()
                        .await
                        .unwrap();
                    if let Some(messages) = messages.messages {
                        write_queue_messages_to_file(messages).unwrap();
                    } else {
                        println!("No messages found.");
                    }
                }
            } else {
                println!("No queues found.");
            }
        }
        Err(error) => {
            eprintln!("Error listing queues: {:?}", error);
        }
    }
    println!("Finished");
}

fn write_queue_messages_to_file(messages: Vec<Message>) -> std::io::Result<()> {
    let file = File::create("output.ndjson")?;
    let mut writer = BufWriter::new(file);

    for message in messages {
        // Use serde to serialize the message to a string
        if let Some(body) = message.body {
            writeln!(&mut writer, "{}", body)?;
        }
    }

    Ok(())
}
