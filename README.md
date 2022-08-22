
# Datalake Hunter

```(shell)
  _____        _        _       _          _    _             _            
 |  __ \      | |      | |     | |        | |  | |           | |           
 | |  | | __ _| |_ __ _| | __ _| | _____  | |__| |_   _ _ __ | |_ ___ _ __ 
 | |  | |/ _` | __/ _` | |/ _` | |/ / _ \ |  __  | | | | '_ \| __/ _ \ '__|
 | |__| | (_| | || (_| | | (_| |   <  __/ | |  | | |_| | | | | ||  __/ |   
 |_____/ \__,_|\__\__,_|_|\__,_|_|\_\___| |_|  |_|\__,_|_| |_|\__\___|_|   
```

A CLI program to create bloom filters from Datalake and check for matches from a list of value, even when offline.
Bloom filters are created using the [bloomfilter](https://crates.io/crates/bloomfilter) crate and saved using the RON format.

## Usage

The program can be used with:

```(shell)
dtl_hunter <command> <parameter>
```

Check `dtl_hunter -h` for help, including a list of commands and flags avaiable.

### Global Options

- `-e` | `--environment` : The Datalake API environment. Default to production. Possible values are `prod`, `preprod`
- `-v` | `--version` :  Prints the installed version.
- `-h` | `--help` : Prints the help message.

## Create command

Allow users to create bloom filters for Datalake Hunter. Using bloom filters, users can search values in Datalake without an internet connection.

Two types of input are available to create bloom filters:

- A file with a value on each line
- A queryhash from Datalake

### Example

Using the following command, a bloom filter named `dangerous_ip.bloom` will be created in the current directory from a text file `dangerous_ip.txt` which contains one value per line. A false positive rate of `0.0001` is used.

```(shell)
dtl_hunter create -f dangerous_ip.txt -o dangerous_ip.bloom -r 0.0001
```

### Options

- `-f` | `--file` : Path to the file to use to create a bloom filter. One value per line.
- `-o` | `--output` : Path to the file to output the created bloom filter.
- `-q` | `--queryhash` : Query hash from which to build a bloom filter.
- `-r` | `--rate` : Rate of false positive. Can be between `0.0` and `1.0`. The lower the rate the bigger the bloom filter will be. Default is `0.00001`.

## Check command

Allow users to checks if values in a provided file can be found in bloom filters or in Datalake using query hashes.

Matched values can also be looked up on Datalake using the `-l` flag with the path to the file in which to save the results. See below for more informations on options.

Multiple bloom filters and query hashes can be provided on a single check.

The output will be saved to in a csv using the following format:

```(csv)
matching_value,bloom_filename
```

When a query hash is provided, it will be used as the name of the bloom filter in the csv file.

## Example

Using the following command, a check will be performed for the values in the input file `input.txt` on each bloom filters and query hashes. The output will be saved in the file `output.csv` in the current directory.

```(shell)
dtl_hunter check -i input.txt -o output.csv -b subfolder/ip.bloom -b very_dangerous_ip.bloom -q 54c871d4c27d6e728a263de238633aad -q 031576cc0c6a2ec525047f0e0fab4181
```

## Options

- `-q` | `--queryhash` : Query hash from which to build a bloom filter. Required if no bloom filter files are provided.
- `-b` | `--bloom` : Path to a bloom filter to be used for the check. Required if no query hashes are provided.
- `-i` | `--input` : Path to file containing the value to check, one value per line.
- `-l` | `--lookup` : Path to the file in which Lookup matched values should be written.
- `-o` | `--output` : Path to file to which the list of matching inputs will be pushed to as a csv file.
