use std::collections::HashSet;
use dotenv::dotenv;
use std::env;
use nostr_sdk::prelude::*;
use tokio::time::{timeout, Duration};

async fn fetch_trusted_authors_for(main: &str, client: &Client) -> HashSet<PublicKey> {
    let mut authors: HashSet<PublicKey> = HashSet::new();

    let filter_contacts = Filter::new().kind(Kind::ContactList);
    let filter_npub = Filter::new().author(PublicKey::parse(main).unwrap());
    let sub_id: SubscriptionId = client.subscribe(vec![filter_contacts, filter_npub], None).await;

    let mut notifications = client.notifications();
    while let Ok(notification) = timeout(Duration::from_secs(5), notifications.recv()).await {
        if let Ok(RelayPoolNotification::Event { subscription_id, event, .. }) = notification {
            if subscription_id == sub_id && event.kind == Kind::ContactList {
                for element in &event.tags {
                    let pubkey = match element {
                        Tag::PublicKey { public_key, .. } => Some(public_key),
                        _ => None,
                    };
                    if let Some(contact) = pubkey {
                        authors.insert(contact.clone());
                    }
                }
            }
        }
    }
    println!("Pullend {} watched pubkeys.", authors.len());
    authors
}

async fn check_and_comment(note_string: &String, event_id: &EventId, client: &Client) {
    // will this ever get finished?
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    
    // Or use your already existing (from hex or bech32)
    let my_keys = Keys::parse(env::var("BOT_NSEC").unwrap())?;

    // Create new client
    let client = Client::new(&my_keys);

    // Add relays
    client.add_relay("wss://relay.damus.io").await?;
    client.add_relay("wss://relay.primal.net").await?;
    client.add_relay("wss://relay.nostr.band").await?;
    client.add_relay("wss://ftp.halifax.rwth-aachen.de/nostr").await?;
    client.add_relay("wss://nostr.mom").await?;
    client.add_relay("wss://relay.nostrplebs.com").await?;

    // Connect to relays
    client.connect().await;

    // let event: Event = EventBuilder::text_note("POW text note from nostr-sdk", []).to_pow_event(&my_keys, 20)?;
    // client.send_event(event).await?; // Send to all relays
    // client.send_event_to(["wss://relay.damus.io"], event).await?; // Send to specific relay
    let authors = fetch_trusted_authors_for(&env::var("TRUSTED_NPUB").unwrap(), &client).await;

    let filter_note = Filter::new().kind(Kind::TextNote);
    let filter_npub = Filter::new().authors(authors);
    let sub_id: SubscriptionId = client.subscribe(vec![filter_note, filter_npub], None).await;

    let mut notifications = client.notifications();

    while let Ok(notification) = notifications.recv().await {
         if let RelayPoolNotification::Event { subscription_id, event, .. } = notification {
             if subscription_id == sub_id && event.kind == Kind::TextNote {
                check_and_comment(&event.content, &event.id, &client);
                break; // Exit
            }
        }
    }

    Ok(())
}