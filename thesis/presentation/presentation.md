ROOC: linguaggio e ambiente integrato per la modellazione e la risoluzione di problemi di ottimizzazione lineare

# Slide 1
Vorrei iniziare proponendovi un problema.

Il vostro nutrizionista vi ha dato una dieta da seguire:
- la dieta richiede di assumere almeno una certa quantità di alcuni macronutrienti
- ma allo stesso tempo di non superarne altri, per esempio un massimo di 50g di grassi

Siete al supermercato e dovete comprare gli ingredienti necessari per rispettare la dieta, spendendo il meno possibile.
Di ogni ingrediente conoscete il costo e i valori nutrizionali per ciascun macronutriente.

Quali ingredienti scegliete?

---

Quello che vi ho appena descritto si chiama "problema della dieta" ed è un esempio classico usato
per introdurre la programmazione lineare.

Questo problema si può scrivere in modo formale tramite un modello di programmazione lineare.

[[mostra a sinistra la funzione obiettivo e i vincoli in linguaggio naturale, in modo che sembrino simili a una modellazione formale ma restando puramente testuali e colloquiali]]

Prima di scrivere il modello ci servono alcune cose: una tabella $T$ che indica quanti macronutrienti assumiamo consumando un certo ingrediente, e un insieme di variabili decisionali $X_i$ che, per ogni ingrediente $i$, ci dicono quante unità di quell'ingrediente acquistiamo. Infine ci servono i valori minimi $Nmin$ e massimi $Nmax$ previsti dalla dieta.

[[mostra a destra un modello matematico di ottimizzazione lineare e collega tramite delle frecce la parte testuale al corrispondente modello matematico]]

Quella che abbiamo appena scritto è la formulazione di programmazione lineare del problema della dieta. Come potete notare, i dati del problema non compaiono all'interno del modello.

---

Il modello descrive come è formulato il problema, mentre un'istanza fornisce i dati necessari affinché il modello possa essere risolto da un solver di programmazione lineare.

In generale, per trovare una soluzione ottima serve uno strumento che esegua una serie di passaggi:

scrittura del modello -> aggiunta dei dati di un'istanza -> conversione nel formato accettato dal solver -> risoluzione tramite solver -> soluzione del problema


---

Nell'ecosistema della programmazione lineare esistono già strumenti in grado di farlo, ma hanno alcune limitazioni:
- sono vincolati all'ecosistema in cui vengono eseguiti
- sono delle black box
- sono difficili da usare per i principianti e poco accessibili

L'obiettivo della mia tesi è risolvere questi tre problemi, realizzando uno strumento che permetta a studenti e persone poco esperte di
risolvere problemi e imparare la modellazione.

---

Torniamo al problema della dieta.

[[sinistra il modello matematico]]

ROOC è un linguaggio che permette di scrivere in modo formale un modello di ottimizzazione lineare.

[[destra il modello riscritto in sintassi ROOC, mostrando come ogni elemento del modello matematico viene tradotto]]

ROOC usa un compilatore per convertire il modello e i dati dell'istanza nel formato utilizzato dai solver.

In pratica implementa i passaggi visti prima come una pipeline, che va dalla scrittura del modello fino al risultato finale:

Model (il modello che abbiamo scritto) -> LinearModel (un modello in forma lineare, facile da convertire nel formato del solver) -> [...conversione nel formato del solver...] -> soluzione ottima

---

ROOC, insieme al suo solver di default microlp (un altro contributo di questa tesi), è stato sviluppato interamente in Rust, un linguaggio che permette di compilare per molti ambienti diversi, tra cui il web e i sistemi embedded.

Inoltre i passaggi intermedi tra il modello e la soluzione sono visualizzabili, così da capire come il modello è stato trasformato.

Mettendo insieme queste due cose è nato un ambiente di sviluppo che gira interamente nel browser, in cui si possono scrivere e risolvere modelli visualizzando tutti i passaggi intermedi.

---

ROOC si comporta come un vero e proprio linguaggio di programmazione: ha un type system e strumenti per il debugging e per verificare la correttezza semantica del modello.

[[serie di screenshot delle funzionalità dell'ambiente di sviluppo]]

Vediamolo in una demo.

---

[[demo del sito con il modello della dieta]]

{... fa vedere delle cose ...}

---

## Linearizzazione

Un'altra funzionalità fondamentale di ROOC è la capacità di linearizzare modelli non lineari. Per esempio:

[[sinistra un qualcosa di non lineare come un vincolo con un valore assoluto, destra il vincolo convertito]]
[[vincolo che usa operatori logici e aritmetici, destra il vincolo convertito]]
[[...altro che potrebbe essere utile...]]

---

## microlp

Un'altra parte importante della tesi è microlp, un fork di minilp, un solver di programmazione lineare il cui sviluppo era stato archiviato dall'autore. A minilp ho aggiunto il supporto alle variabili intere, trasformandolo in un solver di programmazione lineare intera mista.

microlp ha avuto un ottimo riscontro ed è diventato uno dei solver di default nell'ecosistema Rust.

---

## Limitazioni

Essendo pensato soprattutto per la facilità d'uso e la didattica, il linguaggio non ha alcune funzionalità presenti in altri linguaggi di modellazione. Tuttavia la sua architettura permette di aggiungerne di nuove in futuro.

## Sviluppi futuri

- Il linguaggio potrebbe supportare i vincoli indicatori, cioè vincoli composti da un'espressione aritmetica e da una condizione logica che li attiva o disattiva, linearizzati tramite la tecnica del big M
- La piattaforma web potrebbe includere una documentazione o un corso interattivo, per insegnare meglio la programmazione lineare e l'uso di ROOC
