# এক বার্তা, সব পর্দা — the website film (storyline C with A's story)

GOAL:    a viewer abroad feels "I can write to Ma in Bangla, on whatever is in my hand", and knows
         Ekushe Next is free on Windows, macOS, Linux and Android.
OUTPUT:  HyperFrames, 1920×1080 at 30 fps (index.html) and 1080×1920 (vertical.html), same scenes.
TIME:    49 s × 30 = 1470 frames. Music only (made with ekushe-promo/music, the films' own score
         library), soft key ticks on typed letters, one whoosh per zoom, a bell when Ma's reply lands.
STORY:   One son, one chat with Ma, one day. Each device types the next line of the same chat
         (the chat syncs, so the earlier lines are already there). The camera never cuts: every
         device ends by diving into the blue suggestion chip, and the next device opens out of a blue
         chip. Ma reads all four lines in Dhaka and answers.
LINES:   the engine's real output (assets/trace.js), checked with demo_trace:
         Windows  ma, kemon acho?                   → মা, কেমন আছো?
         Mac      aj notun kaj shuru holo.          → আজ নতুন কাজ শুরু হলো।
         Android  tomar haater ranna khub miss kori. → তোমার হাতের রান্না খুব মিস করি।
         Tablet   shiggiri deshe ashbo, inshaAllah.  → শিগগিরই দেশে আসব, ইনশাআল্লাহ।
         Ma       tumi bhalo theko baba.            → তুমি ভালো থেকো বাবা।
LOOK:    each world one flat ground (no gradients, no glow): night #142130, office #ECE7DE, bus at
         sunset #E9A273, home evening #27463A, Dhaka morning #F4E6CB; the end card is Ekushe green
         #06372A with the red sun of the launch film. Ink #FFFFFF on dark grounds, #1D1D1F on light.
         The one accent is the suggestion blue (#3C7BF0 light, #8AB4F8 dark): the chosen chip, the
         sent bubble, the end-card button. Titles Hind Siliguri 700 at 64 px (6% of 1080), the
         English line Inter Tight 500 at 30 px. Devices fill 55–65% of the frame height.
OBJECTS: a laptop (screen in a black bezel on a silver base), a MacBook, an Android phone held
         upright, a tablet on a keyboard cover, Ma's phone; each shows the chat with Ma and
         Ekushe's real bar for that system (Windows frosted dark, Mac frosted light, Android strip
         over the real Ekushe Default keys, the tablet's strip above its desktop taskbar).
MOTION:  pull-back out of a chip 1.4 s expo.out (scale 14 → 1); zoom into a chip 0.7 s power3.in
         (1 → 16); titles land with a per-word blur (6 px → 0, 0.13 s, 0.07 s apart) and hold still;
         typing 3 frames a letter, 6 frames between words.
SHOTS:   0–8 s Windows at night · 8–16 Mac at the office · 16–24 Android on the bus ·
         24–32 tablet with a keyboard at home · 32–42 Ma in Dhaka reads, replies; "মায়ের ভাষায়,
         সব পর্দায়" · 42–49 end card: the reply's send button opens into the logo, name, platforms,
         ekushenext.com.
BANNED:  dark void with small floating things; glow, particles, lens flares; gradients; cards in a
         row; corner labels; titles that drift after landing; the logo docked in a corner; invented
         UI claiming to be Ekushe's (the bars and keys are the apps' own).
QA:      2 s, 7 s (chip fills), 12 s, 20 s, 28 s, 36 s, 40 s, 46 s, 48.5 s; the vertical at the same.
