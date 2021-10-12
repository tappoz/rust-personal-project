use log;

use crate::factory;
use crate::model;

use amiquip::{AmqpProperties, ConsumerMessage, ConsumerOptions, Exchange, Publish};

// TODO move this to config files...
pub const QUEUE_CONNECTION_STR: &'static str = "amqp://guest:guest@localhost:5672";
pub const QUEUE_NAME: &'static str = "pp_work_queue";

fn publish_msg(work: &model::WorkDemand, exchange: &Exchange) -> Result<(), String> {
    // serialize the input structure
    let work_json_str = serde_json::to_string(work).unwrap();

    log::info!(
        "Sending message '{}' to queue {}",
        work_json_str,
        QUEUE_NAME
    );
    match exchange.publish(Publish::with_properties(
        work_json_str.as_bytes(),
        QUEUE_NAME,
        // delivery_mode 2 makes work_json_str persistent
        AmqpProperties::default().with_delivery_mode(2),
    )) {
        Ok(val) => log::info!("Sent message '{}': {:?}", work_json_str, val),
        Err(err) => {
            let err_msg = format!("Couldnt send message '{}': {}", work_json_str, err);
            log::error!("{}", err_msg);
            return Err(err_msg);
        }
    };

    Ok(())
}

pub fn publish_all(works: Vec<&model::WorkDemand>) -> Result<(), Vec<String>> {
    let mut connection = factory::amqp_connection();
    let channel = connection.open_channel(None).unwrap();
    let exchange = Exchange::direct(&channel);
    let channel_id = channel.channel_id();
    log::info!(
        "Direct connection to AMQP echange via channel with ID {}",
        channel_id
    );

    // best effort: dispatch what you can
    let mut dispatched_counter = 0;
    let mut errs: Vec<String> = Vec::new();
    for work in works {
        match publish_msg(work, &exchange) {
            Ok(_) => {
                dispatched_counter += 1;
            }
            Err(err) => {
                errs.push(err);
            }
        }
    }
    if errs.len() > 0 {
        log::error!(
            "We were not able to dispatch all messages, only {}",
            dispatched_counter
        );
        return Err(errs);
    }

    match channel.close() {
        Ok(val) => log::info!("Closed AMQP channel with ID {}: {:?}", channel_id, val),
        Err(err) => {
            let err_msg = format!(
                "Couldn't close AMQP channel with ID {}: {:?}",
                channel_id, err
            );
            errs.push(err_msg);
            return Err(errs);
        }
    };

    match connection.close() {
        Ok(val) => log::info!("Closed AMQP connection: {:?}", val),
        Err(err) => {
            let err_msg = format!("Couldn't close AMQP connection: {:?}", err);
            errs.push(err_msg);
            return Err(errs);
        }
    };

    Ok(())
}

// https://github.com/jgallagher/amiquip/blob/master/examples/work_queues_new_task.rs
pub fn publish(work: &model::WorkDemand) -> Result<(), String> {
    let mut connection = factory::amqp_connection();
    let channel = connection.open_channel(None).unwrap();
    let exchange = Exchange::direct(&channel);
    let channel_id = channel.channel_id();
    log::info!(
        "Direct connection to AMQP echange via channel with ID {}",
        channel_id
    );

    match publish_msg(work, &exchange) {
        Ok(_) => log::info!(
            "Successfully published message {:?} on AMQP channel ID {}",
            work,
            channel_id
        ),
        Err(err) => {
            let err_msg = format!("{}", err);
            return Err(err_msg);
        }
    };

    match channel.close() {
        Ok(val) => log::info!("Closed AMQP channel with ID {}: {:?}", channel_id, val),
        Err(err) => {
            let err_msg = format!(
                "Couldn't close AMQP channel with ID {}: {:?}",
                channel_id, err
            );
            return Err(err_msg);
        }
    };

    match connection.close() {
        Ok(val) => log::info!("Closed AMQP connection: {:?}", val),
        Err(err) => {
            let err_msg = format!("Couldn't close AMQP connection: {:?}", err);
            return Err(err_msg);
        }
    };

    Ok(())
}

// TODO
// queue_declare, then message count
// https://github.com/jgallagher/amiquip/blob/3a9849189d0694d2421cc39c1770276cde4905f0/examples/pubsub_receive_logs.rs
// then return both the `n` messages and the message count!

// https://github.com/jgallagher/amiquip/blob/master/examples/work_queues_worker.rs
// https://docs.rs/amiquip/0.4.0/amiquip/struct.Consumer.html
pub fn consume_amqp_queue(n: usize) -> Result<Vec<model::WorkDemand>, Vec<String>> {
    let mut connection = factory::amqp_connection();
    let channel = connection.open_channel(None).unwrap();
    let channel_id = channel.channel_id();

    let mut errs: Vec<String> = Vec::new();

    // Set QOS to only send us 1 message at a time.
    match channel.qos(0, 1, false) {
        Ok(val) => log::info!(
            "Done setting up AMQP channel with ID {}: {:?}",
            channel_id,
            val
        ),
        Err(err) => {
            let err_msg = format!("Couldnt setup AMQP channel with ID {}: {}", channel_id, err);
            log::error!("{}", err_msg);
            errs.push(err_msg);
            return Err(errs);
        }
    };

    let consumer = channel
        .basic_consume(
            QUEUE_NAME,
            ConsumerOptions {
                no_ack: true,
                ..ConsumerOptions::default()
            },
        )
        .unwrap();

    let mut works: Vec<model::WorkDemand> = Vec::new();
    // https://github.com/jgallagher/amiquip/blob/master/examples/hello_world_consume.rs
    for (i, message) in consumer.receiver().iter().enumerate() {
        match message {
            ConsumerMessage::Delivery(delivery) => {
                let queue_msg = String::from_utf8_lossy(&delivery.body).to_string();
                // desirializing
                log::info!("Loop ID {:>3} - parsing message JSON: {}", i, queue_msg);
                let work_from_json: model::WorkDemand =
                    serde_json::from_str(queue_msg.as_str()).unwrap();
                works.push(work_from_json);
                // match channel.ack_all()
                match consumer.ack(delivery) {
                    Ok(val) => log::info!("Consumed and ACK AMQP delivery: {:?}", val),
                    Err(err) => {
                        let err_msg = format!("Couldnt consume and ACK AMQP delivery: {}", err);
                        log::error!("{}", err_msg);
                        errs.push(err_msg);
                    }
                };
            }
            other => {
                println!("Consumer ended: {:?}", other);
                break;
            }
        }
        if i >= n {
            log::info!(
                "Done with the batch, so far parsed these {} messages: {:?}",
                works.len(),
                works
            );
            match consumer.cancel() {
                Ok(val) => {
                    log::info!(
                        "Cancelled the receiving process via AMQP channel with ID {}: {:?}",
                        channel_id,
                        val
                    )
                }
                Err(err) => {
                    let err_msg = format!(
                        "Couldnt cancel the receiving process via AMQP channel with ID {}: {}",
                        channel_id, err
                    );
                    log::error!("{}", err_msg);
                    errs.push(err_msg);
                    return Err(errs);
                }
            };
        }
    }

    // TODO do something about connections e.g. sharing them...
    match connection.close() {
        Ok(val) => log::info!("Closed AMQP connection: {:?}", val),
        Err(err) => {
            let err_msg = format!("Couldnt close AMQP connection: {}", err);
            log::error!("{}", err_msg);
            errs.push(err_msg);
            return Err(errs);
        }
    };

    // return the works...
    log::info!("Returning a list of {} messages...", works.len());
    Ok(works)
}
