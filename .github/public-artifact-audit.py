"""Supplementary artifact scan; reports remain encrypted, no credential verification."""
import ast
import base64
import gzip
import json
import os
from pathlib import Path
import subprocess
import tempfile
import urllib.request
import urllib.error

repo = os.environ['AUDIT_REPO']
assert repo in ('uPack-dev/strapi-nuxt-ua-template', 'netherg-io/create-fabric', 'netherg-io/simplebedrockmodel-fabric', 'netherguy4/audio-integrity')
root = Path(tempfile.mkdtemp(prefix='artifact-secret-review-'))
inputs = root/'inputs'
inputs.mkdir()
report = {'repository': repo, 'listed': [], 'downloaded': [], 'skipped': [], 'errors': [], 'findings': []}
headers = {'User-Agent': 'owner-requested-secret-audit', 'Accept': 'application/vnd.github+json', 'Authorization': 'Bearer ' + os.environ['AUDIT_READ_TOKEN']}

def get(endpoint, limit):
    try:
        with urllib.request.urlopen(urllib.request.Request('https://api.github.com/repos/'+repo+'/'+endpoint, headers=headers), timeout=45) as response:
            data=response.read(limit+1)
            if len(data)>limit:
                raise ValueError('size limit')
            return data
    except urllib.error.HTTPError as error:
        report['errors'].append({'endpoint': endpoint, 'http_status': error.code})
    except Exception as error:
        report['errors'].append({'endpoint': endpoint, 'error_type': type(error).__name__})
    return None

for page in range(1,21):
    data=get(f'actions/artifacts?per_page=100&page={page}', 12*1024*1024)
    if data is None:
        report['listing_complete']=False
        break
    page_items=json.loads(data)['artifacts']
    report['listed'] += [{'id':a['id'],'name':a['name'],'bytes':a['size_in_bytes'],'expired':a['expired']} for a in page_items]
    if len(page_items)<100:
        report['listing_complete']=True
        break
else:
    report['listing_complete']=False

size=0
for artifact in report['listed']:
    if artifact['expired'] or artifact['bytes']>64*1024*1024 or size+artifact['bytes']>384*1024*1024:
        report['skipped'].append(artifact)
        continue
    data=get('actions/artifacts/'+str(artifact['id'])+'/zip', 65*1024*1024)
    if data is None:
        report['skipped'].append(artifact)
        continue
    (inputs/('artifact-'+str(artifact['id'])+'.zip')).write_bytes(data)
    report['downloaded'].append(artifact)
    size+=len(data)
report['downloaded_bytes']=size
image='ghcr.io/gitleaks/gitleaks@sha256:c00b6bd0aeb3071cbcb79009cb16a60dd9e0a7c60e2be9ab65d25e6bc8abbb7f'
(root/'default.toml').write_text('[extend]\nuseDefault = true\n')
(root/'empty.ignore').write_text('')
p=subprocess.run(['docker','pull','--quiet',image],stdout=subprocess.PIPE,stderr=subprocess.PIPE,timeout=120)
if p.returncode:
    report['errors'].append({'docker_pull_exit':p.returncode})
else:
    command=['docker','run','--rm','--network','none','--user',str(os.getuid())+':'+str(os.getgid()),'-v',str(root)+':/audit',image,'dir','/audit/inputs','--config','/audit/default.toml','--gitleaks-ignore-path','/audit/empty.ignore','--ignore-gitleaks-allow','--max-archive-depth','5','--max-decode-depth','3','--max-target-megabytes','128','--no-banner','--no-color','--log-level','error','--report-format','json','--report-path','/audit/findings.json','--exit-code','17']
    try:
        p=subprocess.run(command,stdout=subprocess.PIPE,stderr=subprocess.PIPE,timeout=480)
        report['scanner_exit']=p.returncode
        report['findings']=json.loads((root/'findings.json').read_text()) if (root/'findings.json').exists() else []
        report['scanner_diagnostics']=p.stderr.decode(errors='replace')[-6000:]
    except Exception as error:
        report['errors'].append({'scan_error_type':type(error).__name__})
cert=None
for node in ast.parse(Path('.github/public-secret-audit.py').read_text()).body:
    if isinstance(node,ast.Assign) and any(isinstance(t,ast.Name) and t.id=='CERT' for t in node.targets):
        cert=ast.literal_eval(node.value)
assert cert
(root/'recipient.pem').write_text(cert)
(root/'report.gz').write_bytes(gzip.compress(json.dumps(report,separators=(',',':')).encode()))
subprocess.run(['openssl','cms','-encrypt','-binary','-aes256','-in',str(root/'report.gz'),'-outform','DER','-out',str(root/'report.cms'),str(root/'recipient.pem')],check=True,stdout=subprocess.DEVNULL,stderr=subprocess.PIPE)
print('ARTIFACT_SUMMARY '+json.dumps({'repository':repo,'listed':len(report['listed']),'downloaded':len(report['downloaded']),'skipped':len(report['skipped']),'scanner_exit':report.get('scanner_exit'),'findings':len(report['findings']),'errors':len(report['errors'])}),flush=True)
print('ENCRYPTED_REPORT_BEGIN',flush=True)
print(base64.b64encode((root/'report.cms').read_bytes()).decode(),flush=True)
print('ENCRYPTED_REPORT_END',flush=True)
