import pathlib, re, subprocess
files = subprocess.check_output(['git','ls-files','*.rs'], text=True).splitlines()
def scrub(s):
    out=[]; i=0; n=len(s)
    while i<n:
        if s.startswith('//',i):
            j=s.find('\n',i); j=n if j<0 else j; out.append(' '*(j-i)); i=j
        elif s.startswith('/*',i):
            j=i+2; depth=1
            while j<n and depth:
                if s.startswith('/*',j): depth+=1; j+=2
                elif s.startswith('*/',j): depth-=1; j+=2
                else: j+=1
            out.append(re.sub(r'[^\n]',' ',s[i:j])); i=j
        elif s[i]=='"' or (s[i]=='r' and re.match(r'r#*"',s[i:])):
            m=re.match(r'r(#+)?"',s[i:]) if s[i]=='r' else None
            end='"'+(m.group(1) or '') if m else '"'; j=i+(len(m.group(0)) if m else 1)
            while j<n:
                if s.startswith(end,j): j+=len(end); break
                if not m and s[j]=='\\': j+=2
                else: j+=1
            out.append(re.sub(r'[^\n]',' ',s[i:j])); i=j
        elif s[i]=="'" and re.match(r"'(?:\\.|[^'\\])'",s[i:]):
            j=i+len(re.match(r"'(?:\\.|[^'\\])'",s[i:]).group()); out.append(' '*(j-i)); i=j
        else: out.append(s[i]); i+=1
    return ''.join(out)
allcode=''; production=''
for f in files:
    code=scrub(pathlib.Path(f).read_text()); allcode+=code+'\n'
    if f.startswith(('tests/','benches/')) or f.endswith(('/tests.rs','/macro_tests.rs')): continue
    production+=re.split(r'#\[cfg\(test\)\]\s*mod tests\s*\{',code,maxsplit=1)[0]+'\n'
print('unsafe blocks', len(re.findall(r'\bunsafe\s*\{',allcode)))
print('allow attributes', len(re.findall(r'#!?\[\s*allow\s*\(',allcode)))
print('non-test unwrap/expect', len(re.findall(r'\.(?:unwrap|expect)\s*\(',production)))
# Exclude entire use declarations, including multiline imports and re-exports.
allcode=re.sub(r'(?m)^\s*(?:pub(?:\([^)]*\))?\s+)?use\s+[^;]*;', '', allcode)
print('as casts', len(re.findall(r'\bas\s+(?:[A-Za-z_]|\*)',allcode)))
