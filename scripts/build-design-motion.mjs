/** Reproducible, original browser-rendered motion templates; no external assets. */
import { createRequire } from 'node:module';
import { mkdtemp, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { resolve, join } from 'node:path';
import { spawnSync } from 'node:child_process';
const require = createRequire(new URL('../apps/web/package.json', import.meta.url));
const { chromium } = require('@playwright/test');
const temporary = await mkdtemp(join(tmpdir(), 'fenix-motion-'));
const browser = await chromium.launch({headless:true});
const scenes = {
  'motion-intro': `<div class="eyebrow">FÉNIX STUDIO / IDEAS IN MOTION</div><h1>Make your<br>next move.</h1><p>A little curiosity. A whole new direction.</p><div class="orbit">${[0,1,2,3].map(i=>`<i style="inset:${i*27}px"></i>`).join('')}</div><footer>ORIGINAL MOTION TEMPLATE <span>01 / 04 SEC</span></footer>`,
  'data-motion': `<div class="eyebrow">FÉNIX STUDIO / DATA STORIES</div><h1>From data<br>to direction.</h1><p>See the bigger picture.</p><div class="bars">${[42,62,50,84,100].map((v,i)=>`<div style="--bar:${v}%;--delay:${i*.15}s"><i></i><small>${['M','T','W','T','F'][i]}</small></div>`).join('')}</div><footer>ILLUSTRATIVE DATA <span>02 / 04 SEC</span></footer>`,
};
try {
  for (const [name,scene] of Object.entries(scenes)) {
    const context=await browser.newContext({viewport:{width:960,height:600},recordVideo:{dir:temporary,size:{width:960,height:600}}});
    const page=await context.newPage();
    await page.setContent(`<html><head><style>*{box-sizing:border-box}body{margin:0;padding:55px;background:${name==='motion-intro'?'#181b30':'#173d33'};color:#f5f1e7;font:18px Arial;overflow:hidden}.eyebrow{font-size:12px;letter-spacing:3px}h1{font-size:88px;line-height:1;letter-spacing:-6px;margin:85px 0 25px;position:relative;z-index:2}p{font-size:20px;opacity:.7}footer{position:absolute;bottom:40px;left:55px;right:55px;font-size:11px;letter-spacing:2px;border-top:1px solid #ffffff40;padding-top:20px;display:flex;justify-content:space-between}.orbit{width:410px;height:410px;position:absolute;right:-40px;top:78px;animation:turn 8s linear infinite}.orbit i{position:absolute;border:21px solid #b9a0ff;border-radius:50%;border-bottom-color:transparent}.bars{position:absolute;right:70px;top:125px;width:370px;height:300px;display:flex;gap:17px;align-items:end}.bars>div{flex:1;height:var(--bar);position:relative}.bars i{display:block;background:#d5e8a0;height:100%;border-radius:9px 9px 0 0;transform-origin:bottom;animation:grow 2s both;animation-delay:var(--delay)}.bars small{position:absolute;bottom:-28px;left:40%;font-size:11px;opacity:.6}@keyframes grow{from{transform:scaleY(.05)}to{transform:scaleY(1)}}@keyframes turn{to{transform:rotate(360deg)}}</style></head><body>${scene}</body></html>`);
    await page.waitForTimeout(4800);
    const video=page.video();await context.close();
    const source=await video.path();
    const result=spawnSync('ffmpeg',['-nostdin','-v','error','-y','-ss','0.35','-i',source,'-t','4','-an','-c:v','libvpx-vp9','-threads','1','-b:v','0','-crf','32',resolve('resources/designs/media',name+'.webm')],{stdio:'inherit'});
    if(result.status!==0)throw Error('Motion encoding failed');
    console.log('Rendered '+name);
  }
} finally {await browser.close();await rm(temporary,{recursive:true,force:true});}
