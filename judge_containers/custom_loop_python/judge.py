import json
import sys
import subprocess
from io import StringIO
line = input()
line = StringIO(line)
data = json.load(line)

with open("submission.py", "w") as f:
    f.write(data["task_data"]["code"])

value = "1"

feedback = None

for i in range(10):
    proc = subprocess.run(["python", "submission.py"],
                          capture_output=True, input=value, encoding="utf-8")
    if proc.returncode != 0:
        feedback = {
            "test_results": [
                {
                    "err": "RTE",
                    "message": "runtime error",
                    "score": 0,
                    "metrics": {
                        "sus": 1000
                    }
                }
            ]
        }
        break
    value = proc.stdout

if feedback is None:
    try:
        value = int(value)
    except ValueError:
        feedback = {
            "test_results": [
                {
                    "err": "WA",
                    "message": "it's not even a number",
                    "score": 0,
                    "metrics": {
                        "sus": 1000000000
                    }
                }
            ]
        }

if feedback is None and value == 1024:
    feedback = {
        "test_results": [
            {
                "err": "OK",
                "message": "accepted",
                "score": 100,
                "metrics": {
                    "sus": 10
                }
            }
        ]
    }

if feedback is None:
    feedback = {
        "test_results": [
            {
                "err": "WA",
                "message": "wrong answer",
                "score": 0,
                "metrics": {
                    "sus": 10000
                }
            }
        ]
    }
print(json.dumps(feedback))
