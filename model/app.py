from sklearn.ensemble import IsolationForest
from sklearn.preprocessing import OneHotEncoder
import numpy as np
import pika, sys, os
import json
medium_list = ["UPI","Netbanking","Card"]
model = IsolationForest(contamination=0.01)  # Adjust contamination parameter based on expected rate of anomalies

def preprocess_transactions(transactions):
    # Extract amounts and medium types from transactions
    amounts = [transaction['amount'] for transaction in transactions]
    mediums = [medium_list.index(transaction['medium']) for transaction in transactions]

    # Convert amounts to numpy array
    amounts_array = np.array(amounts).reshape(-1, 1)
    
    mediums_array = np.array(mediums).reshape(-1, 1)

    # Encode medium types using one-hot encoding
    # encoder = OneHotEncoder()
    # mediums_encoded = encoder.fit_transform(np.array(mediums).reshape(-1, 1)).toarray()

    # Concatenate amounts and encoded medium types
    features = np.concatenate([amounts_array, mediums_array], axis=1)
    print(features)
    return features

def detect_anomalies(transactions):
    # Preprocess transactions
    features = preprocess_transactions(transactions)

    # Fit isolation forest model
    model.fit(features)

    # Predict anomalies
    anomalies = model.predict(features)

    return anomalies

# Example transaction data
transactions = [
    {
        "from": {"name": "Raj", "acc_no.": 634567654, "balance": 9000},
        "to": {"name": "Zubair", "acc_no.": 634567623, "balance": 7000},
        "medium":"UPI",
        "amount": 90,
        "transaction_id": 5679864356780972,
    },
    {
        "from": {"name": "Raj", "acc_no.": 634567654, "balance": 9000},
        "to": {"name": "Zubair", "acc_no.": 634567623, "balance": 7000},
        "medium":"UPI",
        "amount": 30,
        "transaction_id": 5679864356780972,
    },
    {
        "from": {"name": "Raj", "acc_no.": 634567654, "balance": 9000},
        "to": {"name": "Zubair", "acc_no.": 634567623, "balance": 7000},
        "medium":"UPI",
        "amount": 60,
        "transaction_id": 5679864356780972,
    },
    {
        "from": {"name": "Raj", "acc_no.": 634567654, "balance": 9000},
        "to": {"name": "Zubair", "acc_no.": 634567623, "balance": 7000},
        "medium":"UPI",
        "amount": 190,
        "transaction_id": 5679864356780972,
    },
    # Add more transactions here
]

# # Detect anomalies
# anomalies = detect_anomalies(transactions)

# # Print anomalies
# for i, anomaly in enumerate(anomalies):
#     if anomaly == -1:
#         print(f"Transaction {i+1} is potentially fraudulent.")



def main():
    connection = pika.BlockingConnection(pika.ConnectionParameters(host='localhost'))
    channel = connection.channel()
    responseChannel=connection.channel()

    responseChannel.queue_declare(queue='amqprs.examples.response', durable=True)
    channel.queue_declare(queue='amqprs.examples.basic', durable=True)
    
    def callback(ch, method, properties, body):
        print(f" [x] Received {body}")
        data=json.loads(body.decode())
        anomalies=detect_anomalies(transactions+[data])
        for i, anomaly in enumerate(anomalies):
            if anomaly == -1 and i==4:
                print(f"Transaction {i+1} is potentially fraudulent.")
                responseChannel.basic_publish('aqmp.topic','aqmp.example', json.dumps({"anomaly":True}))

        if anomaly!=-1 and i==4:
            responseChannel.basic_publish('aqmp.topic','aqmp.example', json.dumps({"anomaly":False}))


    channel.basic_consume(queue='amqprs.examples.basic', on_message_callback=callback, auto_ack=True)

    print(' [*] Waiting for messages. To exit press CTRL+C')
    channel.start_consuming()

if __name__ == '__main__':
    try:
        main()
    except KeyboardInterrupt:
        print('Interrupted')
        try:
            sys.exit(0)
        except SystemExit:
            os._exit(0)