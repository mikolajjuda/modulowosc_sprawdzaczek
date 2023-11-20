import json, sys;
from io import StringIO
line = input()
line = StringIO(line)
data = json.load(line)

feedback = {
    "test_results":[
        {
            "err": "OK",
            "message": "TODO",
            "score": 0,
            "metrics": {
                "sus_factor": 1000
            }
        }
    ]
}
print(json.dumps(feedback))
