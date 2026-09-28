"""Keep upstream judging rules, adapting unsupported transport defaults only."""
import json
import os
import runpy
import sys
import litellm

# Upstream fixes temperature=0 at the LiteLLM layer. Reasoning models reject
# that value before extra_body is merged. Drop unsupported defaults, retaining
# the explicitly configured temperature=1 and reasoning_effort in extra_body.
litellm.drop_params = True

if __name__ == '__main__':
    if sys.argv[1:] == ['--check']:
        from gaia2_cli.judge.engine import create_litellm_engine
        create_litellm_engine(model=os.environ['GAIA2_JUDGE_MODEL'],
            provider=os.environ['GAIA2_JUDGE_PROVIDER'],
            base_url=os.environ['GAIA2_JUDGE_BASE_URL'],
            api_key=os.environ['GAIA2_JUDGE_API_KEY'],
            extra_body=json.loads(os.environ.get('GAIA2_JUDGE_EXTRA_BODY', '{}')),
            validate=True)
        print('Judge transport validated')
    else:
        runpy.run_module('gaia2_cli.daemon.eventd', run_name='__main__')
