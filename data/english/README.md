# English words for the Android keyboard

`words.txt` lists the 50,000 most common English words, most common first, one per line. The
Android keyboard uses it to complete words as you type and to catch typos when the phone's own
spell checker isn't available. `next.tsv` is a small hand-written table of likely next words.

`words.txt` is made with [wordfreq](https://github.com/rspeer/wordfreq) 3.1 (`top_n_list('en', …)`,
keeping plain words and common contractions). Because that data comes from real text, it
also holds common misspellings (teh, recieve, thier); only words the Hunspell en_US dictionary
(from SCOWL) accepts are kept, in its spelling (London, not london), plus a short list of everyday
words it lacks (ok, wifi, YouTube, Dhaka, inshallah…). wordfreq's data is licensed under
[CC BY-SA 4.0](https://creativecommons.org/licenses/by-sa/4.0/), like Ekushe Next's own dictionary;
it is by Robyn Speer and builds on sources including Wikipedia, OpenSubtitles, Reddit, Twitter and
Google Books Ngrams.
