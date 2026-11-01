import re,unicodedata,glob,sys,os
from rapidfuzz.distance import Levenshtein
from collections import Counter
sys.path.insert(0,'scripts'); from correct import load_lex
freq=load_lex()
def norm(s):
    s=unicodedata.normalize("NFC",s).replace("‌","").replace("‍","");return re.sub(r"\s+"," ",s).strip()
run=sys.argv[1]; cat=Counter(); ex={}
for f in glob.glob(f"runs/{run}/*.txt"):
    if f.endswith('.B.txt'): continue
    b=os.path.basename(f)[:-4].rsplit('_',1)[0]
    gt=norm(open(f"eval/{b}.gt.txt").read()).split(); hy=norm(open(f).read()).split()
    for op in Levenshtein.opcodes(gt,hy):
        if op.tag=='equal': continue
        g=gt[op.src_start:op.src_end]; h=hy[op.dest_start:op.dest_end]
        if op.tag=='delete': k='deleted words'
        elif op.tag=='insert': k='inserted junk'
        elif len(g)!=len(h): k='split/merge'
        else:
            for a,bb in zip(g,h):
                ab=re.sub(r'[^ঀ-৿]','',a)
                if re.search(r'[A-Za-z]',bb): k='latin junk'
                elif ab not in freq: k='gt word not in lexicon'
                elif re.sub(r'[^ঀ-৿]','',bb) in freq: k='real-word error'
                else: k='non-word error'
                cat[k]+=1; ex.setdefault(k,[]).append(f"{a}->{bb}")
            continue
        cat[k]+=len(g) or len(h); ex.setdefault(k,[]).append(f"{' '.join(g)}->{' '.join(h)}")
for k,n in cat.most_common(): print(n,k,'|',' ; '.join(ex[k][:12]))
