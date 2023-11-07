# status

Status application written in rust.

This program is for running tests, and check the status of different test executables you write.

Communication is done over ZeroMQ.

## Getting started

You need to have rust installed to run this

```bash
$ git clone https://github.com/D3faIt/status.git
$ cd status
$ cargo run
```

Now it won't really do anything out of the box without a `settings.json` file

## Setting up tests

To get started you need to make a `settings.json`.

In this file, you define what executables should be run to test the status of

Template/Example:

```json
{
  "protocol": "tcp",
  "port": 5747,
  "interval": 600,
  "timeout": 15.0,
  "pause_on_no_internet": true,
  "services": [
    {
      "name": "website_1",
      "command": "commands/test_website_routing.py",
      "args": [
        "--my-arg", "argument"
      ]
    },
    {
      "name": "website_2",
      "command": "commands/web2.sh"
    },
    {
      "name": "vps",
      "command": "commands/vps.py",
      "timeout": 45
    },
    {
      "name": "something",
      "command": "/path/to/my/executable"
    }
  ]
}
```

### Creating tests

The output of these tests **must** return one of these patterns:


| Pattern                  | Description                                                                                                                                                                                      |
|--------------------------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| Status code              | The simplest form the status checker checks for a success or not is by pure status return code. If this is != 0, then it failed.                                                                 |
| Name-success-description | Displays a plain text result split by newline. First being name, second being if it succeeded, last is the description.<br/>To distinguish between multiple tests, you can use 2 newlines `\n\n` |
| JSON output              | The last way of writing a test script is using JSON. In the JSON you are required to have a `name`, `success` and `result`.                                                                      |

**Status code example:**

```bash
$ ./path/to/program
$ echo $?
0  # Success
```

**Name-success-description example:**

| Key     | Type                            | Description                                                                                                                                                    |
|---------|---------------------------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------|
| name    | str                             | Providing the name of the test                                                                                                                                 |
| success | bool, int<0,1>, float<0.0, 1.0> | How successful was the test. Was it `false`/`0` or `true`/`1`.<br/>or maybe something within the test was wrong, and so you can use float in that case instead |
| result  | str                             | Some description of the result,<br/>can be as long as you want till it reached the end or hits `\n\n` which indicates a new test                               |

Status code gets ignored if it finds this pattern.

```text
$ ./path/to/program
postgres
1
Postgres is up

web
true
It works

elasticsearch
1.0
it's up

bot
0.5
Bot is partially running!
This is not good
```

**JSON output example:**

Similarly to Name-success-description, the JSON output also requires these keys. The `result` key can be whatever type supported by JSON.


| Key     | Type                            | Description                                                                                                                                                    |
|---------|---------------------------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------|
| name    | str                             | Providing the name of the test                                                                                                                                 |
| success | bool, int<0,1>, float<0.0, 1.0> | How successful was the test. Was it `false`/`0` or `true`/`1`.<br/>or maybe something within the test was wrong, and so you can use float in that case instead |
| result  | json                            | JSON object, number, string, bool, float, whatever supported by JSON.                                                                                          |

```bash
$ ./test_vps.py  
```

```json
[
  {
    "name": "test_hetzner_vps",
    "success": true,
    "result": 200
  },
  {
    "name": "test_ovh_vps",
    "success": false,
    "result": 500
  }
]
```

An example with different `result` type:

```bash
$ ./test_website_routing.py 
```

```json
{
  "name": "test_redirects_for_example_org",
  "success": true,
  "result": {
    "ip": "82.153.138.97",
    "return_code": 200,
    "ping": 643,
    "redirects": "example.org => https://example.org/"
  }
}
```
*Just outputting the object (not array) is also valid, it will be handled as a single item test*

Lastly, this format is also valid, putting the name as the key:
```json
{
  "test_hetzner_vps": {
    "success": true,
    "result": 200
  },
  "test_ovh_vps": {
    "success": false,
    "result": 500
  }
}
```

## Communicating

Communicating is done over ZeroMQ. There is an example script you can use in the [daemon](/daemon) folder.

```bash
$ ./main.py version
$ ./main.py help
$ ./main.py settings
$ ./main.py service
$ ./main.py service website_1 website_2 vps something
```

Expected responds for checking services:
```json
[
  {
    "name": "website_1",
    "command": "commands/test_website_routing.py",
    "successes": 1.0,
    "result": [
      {
        "name": "example.org",
        "success": 1.0,
        "result": {
          "ip": "82.153.138.97",
          "ping": 643,
          "redirects": "example.org => https://example.org/",
          "return_code": 200
        }
      }
    ]
  },
  {
    "name": "website_2",
    "command": "/path/to/program",
    "successes": 0.875,
    "result": [
      {
        "name": "postgres",
        "success": 1.0,
        "result": "Postgres is up"
      },
      {
        "name": "web",
        "success": 1.0,
        "result": "It works"
      },
      {
        "name": "elasticsearch",
        "success": 1.0,
        "result": "it's up"
      },
      {
        "name": "bot",
        "success": 0.5,
        "result": "Bot is partially running!\nThis is not good"
      }
    ]
  },
  {
    "name": "vps",
    "command": "commands/vps.py",
    "successes": 0.0,
    "result": "signal: 15 (SIGTERM)"
  },
  {
    "name": "something",
    "command": "/path/to/my/executable",
    "successes": 1.0,
    "result": "yeah\n"
  }
]
```

# [TODO list](/todo.org)
