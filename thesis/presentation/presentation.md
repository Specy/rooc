# Slide 1: Copertina

ROOC: linguaggio e ambiente integrato per la modellazione e la risoluzione di problemi di ottimizzazione lineare

---

# Slide 2: Partiamo da un problema

Vorrei iniziare proponendovi un problema.

[[step 1]]
Il vostro nutrizionista vi ha dato una dieta da seguire:
- la dieta richiede di assumere almeno una certa quantità di alcuni macronutrienti
- ma allo stesso tempo di non superarne altri, per esempio un massimo di 70 g di grassi

[[step 2]]
Siete al supermercato e dovete comprare gli ingredienti necessari per rispettare la dieta, spendendo il meno possibile.
Di ogni ingrediente conoscete il costo e i valori nutrizionali per ciascun macronutriente.

[[step 3]]
Quali ingredienti scegliete?

---

# Slide 3: Il problema della dieta

Quello che vi ho appena descritto si chiama "problema della dieta" ed è un esempio classico usato
per introdurre la programmazione lineare.

Questo problema si può scrivere in modo formale tramite un modello di programmazione lineare.

[[step 1]]
[[mostra a sinistra la funzione obiettivo e i vincoli in linguaggio naturale, in modo che sembrino simili a una modellazione formale ma restando puramente testuali e colloquiali]]

[[step 2]]
Prima di scrivere il modello ci servono alcune cose: una tabella $T$ che indica quanti grammi di ogni nutriente ci sono in una porzione di ogni ingrediente, il costo $C_i$ di ogni ingrediente e i valori minimi $Nmin$ e massimi $Nmax$ previsti dalla dieta. Infine ci serve un insieme di variabili decisionali $x_i$ che, per ogni ingrediente $i$, ci dicono quante porzioni di quell'ingrediente acquistiamo.

[[step 3]]
[[mostra a destra un modello matematico di ottimizzazione lineare e collega tramite delle frecce la parte testuale al corrispondente modello matematico]]

[[step 4]]
Quella che abbiamo appena scritto è la formulazione di programmazione lineare del problema della dieta. Come potete notare, i dati del problema non compaiono all'interno del modello: ci sono solo i loro nomi.

---

# Slide 4: Modello e istanza

[[step 1]]
Il modello descrive come è formulato il problema,

[[step 2]]
mentre un'istanza fornisce i dati necessari affinché il modello possa essere risolto da un solver di programmazione lineare.

[[step 3]]
Mettendo insieme modello e istanza otteniamo il modello compilato: le sommatorie vengono espanse e al posto dei nomi dei dati compaiono i loro valori. È questo che un solver sa risolvere.

[[step 4]]
In generale, per trovare una soluzione ottima serve uno strumento che esegua una serie di passaggi:

scrittura del modello -> aggiunta dei dati di un'istanza -> conversione nel formato accettato dal solver -> risoluzione tramite solver -> soluzione del problema

---

# Slide 5: Gli strumenti esistenti

Nell'ecosistema della programmazione lineare esistono già strumenti in grado di farlo, ma hanno alcune limitazioni:
- sono vincolati all'ecosistema in cui vengono eseguiti
- sono delle black box
- sono difficili da usare per i principianti e poco accessibili

[[step 1]]
L'obiettivo della mia tesi è risolvere questi tre problemi, realizzando uno strumento che permetta a studenti e persone poco esperte di
risolvere problemi e imparare la modellazione.

---

# Slide 6: Il linguaggio ROOC

Torniamo al problema della dieta.

[[step 1]]
[[sinistra il modello matematico]]

ROOC è un linguaggio che permette di scrivere in modo formale un modello di ottimizzazione lineare.

[[step 2]]
[[destra il modello riscritto in sintassi ROOC, con le frecce che collegano ogni elemento del modello matematico alla sua traduzione]]

Ogni parte del modello ha il suo corrispettivo: la funzione obiettivo, i vincoli dopo `s.t.`, i dati dell'istanza nel blocco `where` e le variabili, con il loro dominio, nel blocco `define`.

---

# Slide 7: ROOC è un compilatore

ROOC usa un compilatore per convertire il modello e i dati dell'istanza nel formato utilizzato dai solver.

In pratica implementa i passaggi visti prima come una pipeline, che va dalla scrittura del modello fino al risultato finale:

[[step 1]] Sorgente (il modello e i dati che abbiamo scritto) ->
[[step 2]] Model (i dati vengono inseriti nel modello e le sommatorie espanse) ->
[[step 3]] LinearModel (un modello in forma lineare, facile da convertire nel formato del solver) ->
[[step 4]] conversione nel formato del solver scelto ->
[[step 5]] soluzione ottima

E la soluzione risponde alla domanda iniziale: 1,37 porzioni di pollo, 4,44 di riso e niente avocado, per una spesa di 4,28 €.

---

# Slide 8: Decisioni architetturali

ROOC, insieme al suo solver di default microlp (un altro contributo di questa tesi), è stato sviluppato interamente in Rust, un linguaggio che permette di compilare per molti ambienti diversi, tra cui il web e i sistemi embedded.

Inoltre i passaggi intermedi tra il modello e la soluzione sono visualizzabili, così da capire come il modello è stato trasformato.

Mettendo insieme queste due cose è nato un ambiente di sviluppo che gira interamente nel browser, in cui si possono scrivere e risolvere modelli visualizzando tutti i passaggi intermedi.

---

# Slide 9: Demo

ROOC si comporta come un vero e proprio linguaggio di programmazione: ha un type system e strumenti per il debugging e per verificare la correttezza semantica del modello.

Vediamolo direttamente nell'editor, con il modello della dieta:

- [[step 1]] **Tipi**: passo con il mouse su `T` e l'editor mostra il tipo, `Integer[][]`
- [[step 2]] **Errori**: scrivo `C * x_i` al posto di `C[i] * x_i` e l'editor segnala subito che `*` non si può applicare a un array e a un numero
- [[step 3]] **Completamento**: inizio a scrivere `le` e l'editor suggerisce `len` con la sua firma
- [[step 4]] **Esecuzione**: premo Run e vediamo i passaggi della pipeline e la soluzione, la stessa di prima

---

# Slide 10: Linearizzazione

Un'altra funzionalità fondamentale di ROOC è la capacità di linearizzare modelli non lineari. Per esempio:

[[step 1]]
- il valore assoluto diventa una variabile ausiliaria con due vincoli

[[step 2]]
- minimo e massimo, quando basta un verso, non richiedono variabili binarie

[[step 3]]
- gli operatori logici su variabili booleane diventano vincoli lineari: `x or y` diventa `x + y >= 1`

---

# Slide 11: Il solver microlp

Un'altra parte importante della tesi è microlp, un fork di minilp, un solver di programmazione lineare il cui sviluppo era stato archiviato dall'autore. A minilp ho aggiunto il supporto alle variabili intere, trasformandolo in un solver di programmazione lineare intera mista.

microlp ha avuto un buon riscontro: ha superato il milione e mezzo di download su crates.io ed è usato da good_lp, la libreria di ottimizzazione più diffusa in Rust, come solver scritto interamente in Rust.

---

# Slide 12: Limitazioni e sviluppi futuri

## Limitazioni

Essendo pensato soprattutto per la facilità d'uso e la didattica, il linguaggio non ha alcune funzionalità presenti in altri linguaggi di modellazione: per esempio un confronto aritmetico non può comparire dentro un'espressione logica. Tuttavia la sua architettura permette di aggiungerne di nuove in futuro.

## Sviluppi futuri

- Il linguaggio potrebbe supportare i vincoli indicatori, cioè vincoli composti da un'espressione aritmetica e da una condizione logica che li attiva o disattiva, linearizzati tramite la tecnica del big M
- La piattaforma web potrebbe includere una documentazione o un corso interattivo, per insegnare meglio la programmazione lineare e l'uso di ROOC

---

# Slide 13: Chiusura

Grazie per l'attenzione.
