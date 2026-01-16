# A simple payment engine

This is a naive implementation of a simple payment engine. It takes a transactions CSV file and outputs accounts as another CSV. 

**Usage**
```
cargo run --release -- transactions.csv > clients.csv
```

**Data format**

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

### Transactions and Clients Indexing
Transactions and clients are indexed by the transaction ID and client ID respectively. We use `std::collections::BTreeMap` instead of `std::collections::HashMap` for transactions and accounts indexing. The hashmap has performance issues for large datasets. Collision resolution for a large number of hash values significantly increases complexity. Therefore, we use BTreeMap has a stable O(log(N)) performance and doesn't depend on the data size.

## Testing

### Unit test
The critical part of the engine is a transactions execution. Engine unit tests are focused on checking transaction sequence in different cases incl. corner cases with expected failures.

### Whole flow test script
The test script tests the engine on a simple transactions sequence that contains all transaction types. It compares an output with a golden control sample.

Running the test:
```
./test.sh
```