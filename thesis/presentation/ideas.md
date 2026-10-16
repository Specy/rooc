ROOC: linguaggio ed ambiente integrato per la modellazione e soluzione di problemi di ottimizzazione lineare

# Slide 1
Vorrei iniziare dandovi un problema.

Volete comprare gli ingredenti di una dieta fatta dal vostro nutrizionista:
- la dieta vi chiede di assumere almeno una certa quantità di macronutrienti 
- ma allo stesso modo, di non superarne altri, per esempio un massimo di 50g di grassi

Vi trovate al supermercato e volete comprare gli ingredienti necessari per soddisfare la dieta, ma volete
spendere il meno possibile. Di ogni di essi si sanno il costo e i valori nutrizionali per ogni macronutriente.

Quali ingredienti scegliete?

--- 

Questo problema che vi ho appena descritto si chiama "Il problema della dieta" ed è un classico esempio utilizzato
per spiegare le formulazioni di programmazione lineare.

Questo problema può essere scritto in maniera formale tramite un modello di programmazione lineare.

[[mostra a sinistra la funzione obiettivo e vincoli in linguaggio naturale, in maniera che sembri simile ad una modellazione formale ma rimanendo puramente in maniera testuale e colloquiale]]

Prima di scrivere questo modello abbiamo bisogno di una tabella $T$ che ci indica quanti macronutrienti assumiamo se viene consumato un certo ingrediente, ed un insieme di variabili decisionali $X_i$ che per ogni ingrediente $i$ ci dice quanti di un certo ingrediente abbiamo scelto per l'aquisto. In fine abbiamo bisogno dei valori minimi $Nmin$ e massimi $Nmax$ della nostra dieta.

[[mostra a destra un modello matematico di ottimizzazione lineare e collega tramite delle freccie, la parte testuale al corrispettivo modello matematico]]

Questo che abbiamo appena scritto è la formulazione di programmazione lineare del problema della dieta. Come potete notare, i dati del nostro problema non sono scritti dentro questo modello. 

--- 

Il modello ci dice come è formulato un problema, un istanza di dati ci da invece i valori necessari al modello per poter esser risolto da un solver di programmazione lineare.

In generale per poter trovare una soluzione ottima al problema è necessario un qualche strumento che esegue una serie di passaggi

scrittura del modello -> aggiunta dei dati di un istanza -> conversione nel formato accettato da un solver -> risoluzione tramite solver -> soluzione al problema


---

Nell'ecosistema della programmazione lineare esistono strumenti capace di farlo ma hanno delle limitazioni
- sono vincolati dall'ecosistema in cui eseguono
- sono delle black box
- sono difficili da utilizzare da principianti e poco accessibili

Lo scopo della mia tesi è quello di risolvere questi 3 problemi e creare un tool che permette a persone poco esperte o studenti di 
risolvere problemi e imparare la modellazione.

---

Torniamo al problema della dieta

[[sinistra il modello matematico]]

ROOC è un linguaggio che permette la scrittura in maniera formale di un modello di ottimizzazione lineare

[[destra il modello riscritto in sintassi rooc, facendo vedere come si mappa ogni cosa del modello matematico]]

ROOC utilizza un compilatore per convertire un modello rooc e i dati dell'istanza, nel formato utilizzato dai solver. 

Implementa i passaggi mostrati prima in una pipe di esecuzione, dalla scrittura del modello al risultato finale

Model (il modello che abbiamo scritto) -> LinearModel (un modello in forma lineare, adatto per poter essere facilmente convertito nel formato del solver) -> [...conversione nel formato del solver...] -> soluzione ottima

--- 

ROOC insieme al solver di default, microlp, che è lavoro secondario della tesi, è stato sviluppato interamente nel linguaggio di programmazione rust, che permette la compilazione in molteplici ambienti, incluso il web ed embedded.

Inoltre, i passaggi intermedi fra modello e soluzione sono visualizzabili, permettendo di comprendere come il modello è stato trasformato. 

Combinando queste due cose, è stato sviluppato un ambiente di sviluppo eseguibile interamente nel browser, che permette la scrittura e risoluzione di modelli, visualizzando tutti i passaggi intermedi.

---

ROOC si comporta come un vero e proprio linguaggio di programmazione. Ha un type system e strumenti che permettono il debugging e verifica della correttezza semantica del modello. 

[[serie di screenshot delle funzionalità dell'ambiente di sviluppo]]

Che possiamo vedere in una demo

---
a
[[demo del sito con il modello della dieta]]

{... fa vedere delle cose ...}

---

## Linearizzazione

Una altra funzionalità fondamentale di ROOC è l'abilità di linearizzare modelli non lineari. Per esempio

[[sinistra un qualcosa di non lineare come un vincolo con un valore assoluto, destra il vincolo convertito]]
[[vincolo che usa operatori logici e aritmetici, destra il vincolo convertito]]
[[...altro che potrebbe essere utile...]]

---

## microlp

Un altra parte importante della tesi è microlp, una fork di minilp, un solver di programmazione lineare archiviato dall'autore, al cui ho aggiunto la possibilità di aggiungere variabili intere, rendendolo un solver di programmazione lineare intera mista. 

Il solver microlp ha avuto un ottimo successo ed è diventato uno dei solver di default nell'ecosistema rust.

---

## Limitazioni

Essendo incentrato principalmente alla facilità d'uso e insegnamento, il linguaggio non ha alcune funzionalità che altri linguaggi di modellazione hanno, ma l'architettura del linguaggio permette di aggiungere in futuro nuove funzionalità 

## Sviluppi futuri 

- Il linguaggio potrebbe essere migliorato per aggiungere la possibilità di scrivere vincoli indicatori, cioè un espressione aritmetica seguita da un operatore logico che la attiva o meno, utilizzanod la tecnica big M
- Nella piattaforma web potrebbe essere aggiunto una documentazione o corso interattivo per insegnare meglio la programmazione lineare e l'utilizzo di rooc
