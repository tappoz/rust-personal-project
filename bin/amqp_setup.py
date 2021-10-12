import logging

import pika

# queues
# https://www.rabbitmq.com/queues.html
# https://www.rabbitmq.com/confirms.html on the consumer ACK
#
# main config
QUEUE_CONNECTION_STR = "amqp://guest:guest@localhost:5672"
QUEUE_NAME = "pp_work_queue"
# RabbitMQ/AMQP extra config
IS_DURABLE = True  # survive reboots of the broker
IS_AUTO_DELETE = False  # delete after consumer cancels or disconnects


def setup_logs():
    # set some verbosity in the AMQP logs
    reqs_log_cfg = logging.getLogger("pika.adapters")
    reqs_log_cfg.propagate = True
    logging.basicConfig(
        datefmt="%T",  # only time
        format="%(asctime)s %(name)s [%(levelname)s]: %(message)s",
        level=logging.INFO,
    )
    pp_log = logging.getLogger("PP-LOG")
    return pp_log


def setup_queues(pp_log):
    pp_log.info("connect to the broker")
    params = pika.URLParameters(QUEUE_CONNECTION_STR)
    connection = pika.BlockingConnection(params)
    channel = connection.channel()
    pp_log.info("make sure the queue is setup")
    channel.queue_declare(queue=QUEUE_NAME, durable=IS_DURABLE, auto_delete=False)
    pp_log.info("DONE: close the connection")
    connection.close()
    pp_log.info("DONE: exit this script")


if __name__ == "__main__":
    our_logger = setup_logs()
    setup_queues(our_logger)
