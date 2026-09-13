"""One-time audit. Never execute repository code or verify discovered credentials.
Findings (including scanner output) leave the runner only as CMS-encrypted data.
The recipient private key is not available to GitHub Actions.
"""
import base64
import collections
import concurrent.futures
import gzip
import hashlib
import io
import json
import os
from pathlib import Path
import re
import subprocess
import tempfile
import urllib.error
import urllib.parse
import urllib.request
import zipfile

REPOS = ('uPack-dev/strapi-nuxt-ua-template', 'netherg-io/create-fabric', 'netherg-io/simplebedrockmodel-fabric', 'netherguy4/audio-integrity')
REPO = os.environ['AUDIT_REPO']
assert REPO in REPOS
AUDIT_BRANCH = 'audit/public-secret-review-20260913'
IMAGE = 'ghcr.io/gitleaks/gitleaks:v8.30.1'
ROOT = Path(tempfile.mkdtemp(prefix='secret-review-'))
REPORTS = ROOT / 'reports'
REPORTS.mkdir()
MIRROR = ROOT / 'repo.git'
BLOBS = ROOT / 'blobs'
BLOBS.mkdir()
EXTRAS = ROOT / 'extras'
EXTRAS.mkdir()
REPORT = {'repository': REPO, 'errors': [], 'coverage': {}, 'gitleaks': {}, 'sensitive_paths': [], 'custom_candidates': [], 'workflow_inventory': {}, 'operational_docs': {}}
CERT = '''-----BEGIN CERTIFICATE-----
MIIEPzCCAqegAwIBAgIUYXdoD6H0xbMxFjkn9DYcq18c580wDQYJKoZIhvcNAQEL
BQAwLzEtMCsGA1UEAwwkRXBoZW1lcmFsIHJlcG9zaXRvcnkgYXVkaXQgcmVjaXBp
ZW50MB4XDTI2MDkxMzEyNTczNloXDTI2MDkxNDEyNTczNlowLzEtMCsGA1UEAwwk
RXBoZW1lcmFsIHJlcG9zaXRvcnkgYXVkaXQgcmVjaXBpZW50MIIBojANBgkqhkiG
9w0BAQEFAAOCAY8AMIIBigKCAYEA0N2G+GsF0+I2/WixiNBFXC0YwYju5Ah5f0Fz
rfF01kAGXOdxpbAedWWwoefVrlh8U/rMXOm6/B9fjqNH6xpDmu5G0rDMTVLvtIO7
8Gm4Zw0sKwFYbYFBgb3+SDF67KRZp10VquJUVAqa4my1I/SIRu0pFuw2oph7rVCr
lSLDBkHrkJSufKrxTDOMBevgjLz0fH3nwzuvPCNyTNd6aQHt4v26KRkDAuvlVyFa
6wsylQcPJvDsqBPrHpO1B6m6U9je0U3Y4z+YUwoytlOEgiXGJLLOMlUy5+34PegV
o4vaOjk33eMR3YKIHpDh5HAUjhRNYz7Wq98yUxA2+4RF9hhLNqaAS4y6YZdKFyDU
lP/h0XOLaFGfHfS4Ge8Ccxjd1RQg/0UPz0jcY9RSUhRcP1ryMB3jqxYH50+nHJy2
texy8UcZZ5abpv2FtQKcwIz53YlBKN2LgpRN8NObZJ8VydiLU3SnKKKDeVd+VWQB
tdvWGsqV8OLIcAgIYHk3vFihoHBhAgMBAAGjUzBRMB0GA1UdDgQWBBSLF3q8iDXC
VpkTCItN98UeXYn1rzAfBgNVHSMEGDAWgBSLF3q8iDXCVpkTCItN98UeXYn1rzAP
BgNVHRMBAf8EBTADAQH/MA0GCSqGSIb3DQEBCwUAA4IBgQDGkOV4zX/Hye44Ycx1
8dEXxUX4/iucsUuQacUt8FACPgzqiSuX0hhG1EcWACJK87iNakS2Qt2PwRIqxVh6
MI5oij72IZZ0Fr4nbLoHtdjdbdJdCpLOxCk0gTnZoyxjuhi9hy1mE98YeiMp3K2i
jw3025nUwipZsqxQ3dNCipXQegY4nXcP0o9KLX6bY9PfzFnKAk5X4WdqeFtqkByR
iio9XGGtAWq7TSQpBRfNIQySk8uWX3HzgIUyl5DhzVlzhKhZmRLowZw4aqxGjPRn
NjzRYY30LhbLq8LSJuBSzllB6JjshlDKN6918cMciLIk2tL3yMzn99BSyIiVWgCR
l9KfR5W7Ymd/cSLKR1vQzRv8eUsXJTyIV8l2tDcJiqQHt/W2ou4DY14Q41VNu8LO
eUi6WtYLpg8HGcHKcKV13ZwpwWJhpXSozDt/I4Q80lOnLIPk2zJAXrr52FT9+j/b
ePlSxezFFPTcK6E/MbzSG2dXNMnkW60vSQosWnKZEwu/YDA=
-----END CERTIFICATE-----
'''

def run(args, cwd=None, timeout=420):
    p = subprocess.run(args, cwd=cwd, stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=timeout)
    if p.returncode:
        # Error output stays inside the encrypted report, never public stdout.
        REPORT['errors'].append({'command': args[:3], 'code': p.returncode, 'detail': p.stderr.decode(errors='replace')[-1800:]})
        raise RuntimeError('subprocess failed')
    return p.stdout

def git(*args):
    return run(['git', '--git-dir=' + str(MIRROR), *args])

def api(path, auth=False, cap=64*1024*1024):
    url = path if path.startswith('https://') else 'https://api.github.com/repos/' + REPO + '/' + path
    headers = {'User-Agent': 'owner-requested-secret-audit', 'Accept': 'application/vnd.github+json'}
    if auth:
        headers['Authorization'] = 'Bearer ' + os.environ['AUDIT_READ_TOKEN']
    try:
        with urllib.request.urlopen(urllib.request.Request(url, headers=headers), timeout=40) as response:
            data = response.read(cap + 1)
            if len(data) > cap:
                raise ValueError('response size limit')
            return data
    except urllib.error.HTTPError as error:
        REPORT['errors'].append({'endpoint': path, 'http_status': error.code})
        return None
    except Exception as error:
        REPORT['errors'].append({'endpoint': path, 'error_type': type(error).__name__})
        return None

def pages(endpoint, field=None, max_pages=20):
    result = []
    for page in range(1, max_pages + 1):
        sep = '&' if '?' in endpoint else '?'
        data = api(endpoint + sep + f'per_page=100&page={page}')
        if data is None:
            return result, False
        value = json.loads(data)
        items = value.get(field, []) if field else value
        if not isinstance(items, list):
            return result, False
        result.extend(items)
        if len(items) < 100:
            return result, True
    return result, False

SENSITIVE = re.compile(r'(?i)(^|/)(\.env(?:\..*)?|id_(?:rsa|dsa|ecdsa|ed25519)|\.npmrc|\.pypirc|\.netrc|credentials(?:\..*)?|secrets?(?:\..*)?|.*\.(?:pem|key|p12|pfx|jks|keystore|sqlite3?|db|sql|bak|backup|log))$')
ASSIGN = re.compile(r'''(?ix)\b([a-z0-9_-]*(?:secret|password|passwd|api[_-]?key|token|salt|app_keys|private[_-]?key)[a-z0-9_-]*)["']?\s*(?:=|:)\s*(["'])(.{1,250}?)\2''')
ENV = re.compile(r'(?im)^\s*([A-Z0-9_]*(?:SECRET|PASSWORD|PASSWD|TOKEN|SALT|API_KEY|APP_KEYS|PRIVATE_KEY)[A-Z0-9_]*)\s*=\s*([^\r\n]+)')
AUTH = re.compile(r'(?i)(?:https?://[^\s/:]+:[^\s/@]+@|authorization["\s:]+(?:bearer|basic)\s+[^\s"\']{8,}|-----BEGIN [A-Z ]*PRIVATE KEY-----|[0-9]{8,12}:[A-Za-z0-9_-]{30,})')
seen_candidates = set()
candidate_total = 0

def inspect_text(text, path, blob=None):
    global candidate_total
    lines = text.splitlines()
    for number, line in enumerate(lines, 1):
        matches = [(m.group(1), m.group(3)) for m in ASSIGN.finditer(line)]
        matches += [(m.group(1), m.group(2).strip()) for m in ENV.finditer(line)]
        matches += [('credential-pattern', m.group(0)) for m in AUTH.finditer(line)]
        if not matches:
            continue
        # Keep source/context encrypted; do not validate any token with a provider.
        for name, value in matches:
            signature = (path, name, value)
            if signature in seen_candidates:
                continue
            seen_candidates.add(signature)
            candidate_total += 1
            if len(REPORT['custom_candidates']) < 220:
                REPORT['custom_candidates'].append({'path': path, 'blob': blob, 'line': number, 'field': name, 'value': value, 'context': '\n'.join(lines[max(0, number-2):number+1])[:1000]})


def scan(mode, source, label, extra=()):
    destination = REPORTS / (label + '.json')
    command = ['docker', 'run', '--rm', '--network', 'none', '--user', str(os.getuid()) + ':' + str(os.getgid()), '-v', str(ROOT)+':/audit', IMAGE, mode, '/audit/' + str(source.relative_to(ROOT)), '--config', '/audit/default.toml', '--gitleaks-ignore-path', '/audit/empty.ignore', '--ignore-gitleaks-allow', '--no-banner', '--no-color', '--log-level', 'error', '--max-archive-depth', '3', '--max-decode-depth', '3', '--max-target-megabytes', '64', '--report-format', 'json', '--report-path', '/audit/reports/' + destination.name, '--exit-code', '17', *extra]
    try:
        p = subprocess.run(command, stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=480)
        findings = json.loads(destination.read_text()) if destination.exists() else []
        if p.returncode not in (0, 17):
            REPORT['errors'].append({'scan': label, 'code': p.returncode, 'detail': p.stderr.decode(errors='replace')[-2400:]})
        compact = []
        keys = set()
        for finding in findings:
            key = (finding.get('File'), finding.get('RuleID'), finding.get('Secret'))
            if key in keys:
                continue
            keys.add(key)
            compact.append({k: finding.get(k) for k in ('RuleID', 'File', 'StartLine', 'EndLine', 'Commit', 'Date', 'Secret', 'Match', 'Fingerprint')})
        REPORT['gitleaks'][label] = {'exit': p.returncode, 'findings_total': len(findings), 'unique_total': len(compact), 'findings': compact[:180], 'findings_truncated': len(compact) > 180}
    except Exception as error:
        REPORT['errors'].append({'scan': label, 'error_type': type(error).__name__})


def source_audit():
    run(['git', 'clone', '--mirror', '--quiet', 'https://github.com/' + REPO + '.git', str(MIRROR)])
    p = subprocess.run(['git', '--git-dir='+str(MIRROR), 'fetch', '--quiet', 'origin', '+refs/pull/*/head:refs/audit-pr/*'], stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=240)
    REPORT['coverage']['pr_refs_fetch_exit'] = p.returncode
    refs = git('for-each-ref', '--format=%(refname) %(objectname)').decode().splitlines()
    for ref in refs:
        if ref.split()[0].startswith('refs/heads/' + AUDIT_BRANCH):
            git('update-ref', '-d', ref.split()[0])
    refs = git('for-each-ref', '--format=%(refname) %(objectname)').decode().splitlines()
    REPORT['coverage']['refs'] = refs
    REPORT['coverage']['commits'] = int(git('rev-list', '--all', '--count'))
    REPORT['coverage']['shallow'] = git('rev-parse', '--is-shallow-repository').decode().strip()
    REPORT['coverage']['main_sha'] = git('rev-parse', 'refs/heads/main').decode().strip()
    main_entries = git('ls-tree', '-r', '-z', 'refs/heads/main').split(b'\0')
    current = {}
    for entry in main_entries:
        if not entry:
            continue
        meta, path = entry.split(b'\t', 1)
        mode, typ, oid = meta.decode().split()
        path = path.decode(errors='replace')
        if typ == 'blob':
            current.setdefault(oid, []).append(path)
        if mode == '160000':
            REPORT['coverage'].setdefault('submodules', []).append(path)
        if path.startswith('.github/workflows/') and typ == 'blob':
            REPORT['workflow_inventory'][path] = git('show', 'refs/heads/main:' + path).decode(errors='replace')
    REPORT['coverage']['main_files'] = sum(map(len, current.values()))
    objects = git('rev-list', '--objects', '--all').decode(errors='replace').splitlines()
    paths = {entry.split(' ', 1)[0]: entry.split(' ', 1)[1] if ' ' in entry else '' for entry in objects}
    check = subprocess.run(['git', '--git-dir='+str(MIRROR), 'cat-file', '--batch-check=%(objectname) %(objecttype) %(objectsize)'], input=('\n'.join(paths)+'\n').encode(), stdout=subprocess.PIPE, check=True).stdout.decode().splitlines()
    blob_ids = []
    large = []
    sizes = {}
    for line in check:
        oid, typ, size = line.split()
        if typ != 'blob':
            continue
        size = int(size)
        sizes[oid] = size
        if size > 64*1024*1024:
            large.append({'blob': oid, 'path': paths[oid], 'bytes': size})
        else:
            blob_ids.append(oid)
    REPORT['coverage']['unique_blobs'] = len(sizes)
    REPORT['coverage']['blob_bytes'] = sum(sizes.values())
    REPORT['coverage']['large_blobs_not_dir_scanned'] = large
    out = subprocess.run(['git', '--git-dir='+str(MIRROR), 'cat-file', '--batch'], input=('\n'.join(blob_ids)+'\n').encode(), stdout=subprocess.PIPE, check=True).stdout
    stream = io.BytesIO(out)
    blob_map = {}
    text_blobs = 0
    for oid in blob_ids:
        header = stream.readline().decode().split()
        size = int(header[2])
        data = stream.read(size)
        assert stream.read(1) == b'\n'
        path = paths[oid]
        name = re.sub(r'[^A-Za-z0-9._-]', '_', Path(path).name)[:100] or 'blob'
        output = BLOBS / (oid + '__' + name)
        output.write_bytes(data)
        blob_map[oid] = {'path': path, 'main_paths': current.get(oid, [])}
        if SENSITIVE.search(path):
            REPORT['sensitive_paths'].append({'path': path, 'blob': oid, 'bytes': size, 'in_main': oid in current})
        if data.startswith(b'version https://git-lfs.github.com/spec/v1'):
            REPORT['coverage'].setdefault('lfs_pointers_not_fetched', []).append(path)
        if b'\0' not in data and size < 2*1024*1024:
            text_blobs += 1
            text = data.decode(errors='replace')
            if not path.endswith(('Cargo.lock', 'pnpm-lock.yaml', 'yarn.lock', 'package-lock.json', 'composer.lock')):
                inspect_text(text, path, oid)
            if oid in current and (path.lower().endswith(('readme.md', '.env.example', 'docker-compose.yml', 'compose.yml'))):
                REPORT['operational_docs'][path] = text[:22000]
    REPORT['coverage']['text_blobs_custom_checked'] = text_blobs
    scan('git', MIRROR, 'history', ['--log-opts=--all --full-history'])
    scan('dir', BLOBS, 'all_blobs')
    for label in ('history', 'all_blobs'):
        for finding in REPORT['gitleaks'].get(label, {}).get('findings', []):
            m = re.search(r'([0-9a-f]{40})__', finding.get('File', ''))
            if m and m.group(1) in blob_map:
                finding['source'] = blob_map[m.group(1)]
    for finding in REPORT['custom_candidates']:
        if finding.get('blob') in blob_map:
            finding['main_paths'] = blob_map[finding['blob']]['main_paths']
    # Include commit messages, which source-diff scanning does not guarantee.
    messages = git('log', '--all', '--format=commit %H%n%B')
    (EXTRAS / 'commit-messages.txt').write_bytes(messages)


def supplemental_audit():
    runs, complete = pages('actions/runs', 'workflow_runs')
    eligible = [r for r in runs if r.get('status') == 'completed' and not r.get('head_branch', '').startswith(AUDIT_BRANCH)]
    REPORT['coverage']['actions_runs_listed'] = len(runs)
    REPORT['coverage']['actions_listing_complete'] = complete
    REPORT['coverage']['actions_logs_eligible'] = len(eligible)
    REPORT['coverage']['actions_log_downloaded'] = []
    REPORT['coverage']['actions_log_unavailable'] = []
    for r in eligible[:40]:
        data = api('actions/runs/' + str(r['id']) + '/logs', auth=True)
        if data is None:
            REPORT['coverage']['actions_log_unavailable'].append(r['id'])
            continue
        try:
            archive = zipfile.ZipFile(io.BytesIO(data))
            if sum(x.file_size for x in archive.infolist()) > 192*1024*1024:
                raise ValueError('uncompressed log size limit')
            for index, item in enumerate(archive.infolist()):
                if item.is_dir():
                    continue
                text = archive.read(item)
                path = f'actions-run-{r["id"]}/{item.filename}'
                (EXTRAS / f'run-{r["id"]}-{index}.txt').write_bytes(text)
                inspect_text(text.decode(errors='replace'), path)
            REPORT['coverage']['actions_log_downloaded'].append(r['id'])
        except Exception as error:
            REPORT['errors'].append({'run_id': r['id'], 'error_type': type(error).__name__})
    REPORT['coverage']['actions_logs_limit_skipped'] = max(0, len(eligible)-40)
    # Artifact inventory only; binary artifacts require independent access/size review.
    artifacts, complete = pages('actions/artifacts', 'artifacts')
    REPORT['coverage']['artifacts'] = [{'id': a['id'], 'name': a['name'], 'expired': a['expired'], 'bytes': a['size_in_bytes']} for a in artifacts]
    REPORT['coverage']['artifact_listing_complete'] = complete
    REPORT['coverage']['artifacts_contents_scanned'] = False
    for endpoint in ('issues?state=all', 'issues/comments', 'pulls/comments', 'releases'):
        items, complete = pages(endpoint)
        REPORT['coverage'][endpoint] = {'count': len(items), 'listing_complete': complete}
        filename = re.sub('[^a-zA-Z]', '-', endpoint) + '.txt'
        text = '\n\n'.join(json.dumps({k: x.get(k) for k in ('number', 'html_url', 'title', 'body', 'tag_name')}, ensure_ascii=False) for x in items)
        (EXTRAS / filename).write_text(text)
        inspect_text(text, endpoint)
    scan('dir', EXTRAS, 'logs_and_discussions')

try:
    (ROOT / 'default.toml').write_text('[extend]\nuseDefault = true\n')
    (ROOT / 'empty.ignore').write_text('')
    run(['docker', 'pull', '--quiet', IMAGE], timeout=120)
    REPORT['coverage']['gitleaks_image_digest'] = run(['docker', 'image', 'inspect', IMAGE, '--format', '{{json .RepoDigests}}']).decode().strip()
    source_audit()
    supplemental_audit()
except Exception as error:
    REPORT['errors'].append({'audit_error_type': type(error).__name__})
finally:
    REPORT['coverage']['custom_candidate_total'] = candidate_total
    REPORT['coverage']['custom_candidates_truncated'] = candidate_total > len(REPORT['custom_candidates'])
    summary = {'repository': REPO, 'commits': REPORT['coverage'].get('commits'), 'unique_blobs': REPORT['coverage'].get('unique_blobs'), 'scanner_completed': {k: v['exit'] in (0, 17) for k,v in REPORT['gitleaks'].items()}, 'errors': len(REPORT['errors'])}
    (ROOT/'recipient.pem').write_text(CERT)
    (ROOT/'report.json.gz').write_bytes(gzip.compress(json.dumps(REPORT, ensure_ascii=False, separators=(',', ':')).encode()))
    subprocess.run(['openssl', 'cms', '-encrypt', '-binary', '-aes256', '-in', str(ROOT/'report.json.gz'), '-outform', 'DER', '-out', str(ROOT/'report.cms'), str(ROOT/'recipient.pem')], check=True, stdout=subprocess.DEVNULL, stderr=subprocess.PIPE)
    print('AUDIT_SUMMARY ' + json.dumps(summary), flush=True)
    print('ENCRYPTED_REPORT_BEGIN', flush=True)
    print(base64.b64encode((ROOT/'report.cms').read_bytes()).decode(), flush=True)
    print('ENCRYPTED_REPORT_END', flush=True)
