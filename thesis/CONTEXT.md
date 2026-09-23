# Tesi ROOC

Glossario dei termini usati nella tesi su ROOC, fissati durante la revisione dei
capitoli. Serve a mantenere la stessa terminologia in tutti i capitoli.

## Linguaggio

**fluent API**:
L'interfaccia Rust con cui un modello viene costruito direttamente nel codice
(`ModelBuilder`, operatori sovraccaricati, macro `vars!` e `constraint!`), in
alternativa al linguaggio testuale. Resta in inglese e non si traduce.
_Evitare_: fluid API, API fluente

**pure Rust**:
Proprietà di un componente scritto interamente in Rust, senza dipendenze
native, e quindi compilabile dal sorgente per qualunque target del compilatore
Rust, WebAssembly compreso. È la proprietà che accomuna microlp e ROOC con i
solver predefiniti, e che rende possibile la piattaforma web. Resta in inglese.
_Evitare_: rust only, "compilabile in un singolo binario"

**strumento di modellazione**:
Termine ombrello per ciò che traduce un problema in un modello per un solver:
comprende i **linguaggi di modellazione** (testuali, come AMPL o il linguaggio di
ROOC) e le **librerie di modellazione** (interne a un linguaggio ospite, come
Pyomo, good_lp o la **fluent API** di ROOC).
_Evitare_: "linguaggio di modellazione" riferito a una libreria come good_lp

**modello misto**:
Modello di programmazione lineare intera mista, con variabili continue accanto a
quelle intere o binarie e una funzione obiettivo, in cui vincoli aritmetici e
logici possono convivere. È il qualificatore che rende vera l'affermazione sul
gap dell'ecosistema Rust: pldag linearizza formule logiche, ma solo su variabili
discrete e senza obiettivo.

## Relazioni

- microlp è **pure Rust**; ROOC è **pure Rust** quando usa i solver predefiniti
  (microlp e Clarabel), mentre i backend nativi opzionali (HiGHS, CBC, SCIP…)
  non lo sono e non compilano in WebAssembly.
- Anche good_lp con microlp è **pure Rust**: la proprietà da sola non distingue
  ROOC da good_lp, che se ne differenzia per l'espressività (vincoli logici e
  linearizzazione).
- La **fluent API** produce lo stesso `Model` del linguaggio testuale ed entra
  nella pipeline dalla linearizzazione: le riformulazioni valgono per entrambe.
- L'affermazione sul gap è sempre formulata "al momento della scrittura" e
  limitata all'ecosistema Rust e ai **modelli misti**: fuori da Rust, AMPL,
  Pyomo (GDP), MiniZinc e Gurobi offrono vincoli logici e riformulazioni.

## Ambiguità segnalate

- "fluid API" è stato usato per indicare la **fluent API**: si tratta dello
  stesso concetto, e il termine corretto è *fluent API*.
- "non si può compilare in un singolo binario" è stato usato per dire che
  AMPL, Pyomo e Gurobi non si integrano in un'applicazione Rust. È impreciso,
  perché Gurobi si può linkare in un binario nativo: la proprietà che manca è
  essere **pure Rust**, cioè non richiedere un runtime esterno (l'interprete
  Python, l'eseguibile AMPL) né librerie native precompilate.
- "non ci sono linguaggi di modellazione per Rust" si riferiva a qualunque
  **strumento di modellazione**, librerie comprese: good_lp e oximo sono
  librerie, non linguaggi.
