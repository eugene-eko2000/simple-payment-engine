# A simple payment engine

This is a naive implementation of a simple payment engine. It takes a transactions CSV file and outputs accounts as another CSV. 

### Usage
```
cargo run --release -- transactions.csv > clients.csv
```

### Data format

Input Example
```
type,client,tx,amount
deposit,1,1,1.0
deposit,2,2,2.0
deposit,1,3,2.0
withdrawal,1,4,1.5
withdrawal,2,5,3.0
```
Output Example
```
client,available,held,total,locked
2,2,0,2,false
1,1.5,0,1.5,false
```

## The program logic

The logic is described in the assessment document.

### Dispute caveat
The described dispute flow assumes that the dispute can be made only on a deposit transaction, not on a withdrawal one.

Let's take a look at this example:

| Transaction | Amount | Available | Held | Total |
| ----------- | ------ | --------- | ---- | ----- |
| Deposit | 100 | 100 | 0 | 100 |
| Dispute | | 0 | 100 | 100 |
| Chargeback | | 0 | 0 | 0 |

Let's see how this flow could look for the withdrawal transaction if we apply the described logic.

| Transaction | Amount | Available | Held | Total |
| ----------- | ------ | --------- | ---- | ----- |
| Deposit | 100 | 100 | 0 | 100 |
| Withdrawal | 70 | 30 | 0 | 30 | 
| Dispute | | -40 | 70 | 30 |
| Chargeback | | -40 | 0 | -40 |


These numbers look incorrect. Withdrawal transaction dispure requires a specific design which isn't present in the assessment description. Therefore, the engine checks the transaction type  explicitly and rejects the dispute on the withdrawal transaction.

Also, we are potentially going to overdraft if there was a withdraw between deposit and dispute. Then, in case of a followed chargeback the account can get a negative total. It is actually a current implementation.

## Implementation Details
### Transactions

Transactions are defined as Rust enum. Each enum value matches a certain transaction type. Transaction supports serde::Deserialize.

### Clients

Clients are defined as Rust structure with balances and locked flag.

### Engine

The engine is responsible for storing clients and transactions and transactions execution.

### Execution Flow
1.  Transactions are read from CSV with csv::Reader.
1.  Each CSV record is parsed into a transaction instance.
3.  The transaction instance is sent to the engine for execution.

## Safety
This engine version isn't thread safe. Thread safety isn't necesssary since the input is csv and there is no parallelism by default. 

## Efficiency
The engine is designed for optimal holding up to 4G transactions.

### Data Streaming
The engine streams source csv instead of loading entire file. It's provided by the CSV file reader.
```
csv::Reader::from_path(filename)
```

### Data Storing
Transactions and clients are stored as maps with keys = transaction ID and client ID respectively. 

We use `std::collections::BTreeMap` instead of `std::collections::HashMap` for transactions and accounts indexing.

The hashmap has performance issues for large datasets. Benchmark shows `~100x` performance drop for hash map of `capacity = u32::MAX` which is equal to max number of transactions that are to be supported by the engine. The issue is in a allocation of a huge memory amount. The BTreeMap allocates memory for each element, so a huge memory allocation problem doesn't hit it. 

Therefore, we use BTreeMap that has a predictable `O(log(N))` performance.

## Testing

### Unit Tests
The critical part of the engine is a transactions execution. Engine unit tests are focused on checking transaction sequence in different cases incl. corner cases with expected failures.

### Whole Flow Test Script
The test script tests the engine on a simple transactions sequence that contains all transaction types. It compares an output with a golden control sample.

Running the test:
```
./test.sh
```