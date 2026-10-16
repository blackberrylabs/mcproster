# BRIEF.md — MCProster

> Author's working notes, in French. Public documentation lives in `README.md` and `docs/`.

Cadrage, ecrit une fois. Revision 1 du 2026-10-16, issue de six questions tranchees une par
une par Alexandre dans la conversation Cowork du 16 octobre (fin de journee, juste apres la
release 0.1.1 de Cleanator), validee dans son ensemble le 2026-10-16. Revision 2 du
2026-10-16 apres l'etape 0 (`00_cadrage/2026-10-16_inventaire_analyse.md`) : Claude Desktop
a deux mecanismes (§4), positionnement face aux gestionnaires existants (§3), outils des
extensions connus sans sonde (§6), tests sur faux home (§7). Toute modification ulterieure
passe par une entree du JOURNAL et une revision numerotee.

## 1. Objet

**MCProster fait l'appel des serveurs MCP d'une machine Linux.** Un developpeur qui utilise
plusieurs assistants IA (Claude Desktop, Claude Code, Cursor, Codex, OpenCode, Continue, VS
Code…) a declare des serveurs MCP dans chacun, dans un fichier different, dans un format
different. Personne n'a de vue d'ensemble. MCProster la donne, sur un ecran :

- **la feuille d'appel** : chaque serveur, dans quels hotes il est declare, avec quelle
  commande ou URL, quelle version, quelle portee (globale ou par projet) ;
- **les constats** : doublons (meme serveur dans N hotes, a N versions), serveurs morts
  (commande ou paquet introuvable, ou sonde en echec), cles d'API en clair dans une
  configuration, acces disque larges (serveur `filesystem` sur `/home` entier), variables
  d'environnement sensibles ;
- **pour chaque constat, quoi faire** : le fichier, la ligne, et la commande de l'hote quand
  elle existe (`claude mcp remove`, `codex mcp remove`…), affichee et copiable, jamais
  executee.

Il **regarde et ne touche pas** : il lit les configurations, il n'en modifie aucune, il
n'ecrit rien dans les dossiers des hotes. Il n'a **aucune sortie reseau**. Les secrets qu'il
lit sont **masques partout** et jamais ecrits.

## 2. Pourquoi ce projet (pour moi)

Besoin reel : sur ma machine, une dizaine d'hotes MCP et je ne sais pas ce qui est declare
ou, ni avec quelles cles. Suite logique de Cleanator (meme public, meme methode, meme
organisation) : « autre question, autre outil » (IDEES.md, 2026-10-16). Vitrine open source
et matiere a contenu ; audience et reconnaissance, pas revenu (gratuit, pas de freemium).

## 3. Public et posture

Developpeur Linux multi-assistants, le meme que Cleanator. Il veut une reponse en une
minute a « qu'est-ce que j'ai, ou, est-ce que c'est propre ? », pas un gestionnaire. La
posture est celle d'un inventaire d'inspecteur : exhaustif, factuel, source, qui ne decide
rien a la place de l'utilisateur.

**L'inspecteur, pas le gestionnaire** (etape 0, 2026-10-16). Il existe des gestionnaires de
configurations MCP — mcpm.sh (1 014★, Python, installe et edite les configs de douze hotes),
mcp-linker (342★, Tauri, boutique et synchronisation d'un clic), AreteDriver/mcp-manager
(CLI Python : health-check, doctor, sync), des synchroniseurs CLI. Tous **ecrivent** dans les
configurations ; aucun ne repond en lecture seule a « qu'est-ce que j'ai, ou, est-ce que
c'est propre ? » avec doublons, versions divergentes, serveurs morts, cles en clair et acces
larges ; aucun n'est une application de bureau en un binaire sans runtime. C'est la place de
MCProster, et le README le dira avec le tableau de `00_cadrage/2026-10-16_inventaire_analyse.md`
(a reverifier a la release). Le public est plus etroit que celui de Cleanator : il faut
avoir plusieurs hotes avec des serveurs declares.

## 4. Contexte a verifier a l'etape 1 (sources datees, comme les regles de Cleanator)

> **Rev. 3 (2026-10-16)** : la table ci-dessous est celle du cadrage ; la reference est desormais
> `hosts/*.toml` et `docs/hosts.md` (19 hotes, 38 emplacements, chacun avec sa source citee).


Chaque hote a sa documentation ; **aucun chemin ni format ci-dessous n'est tenu pour acquis
avant d'avoir ete lu dans sa source et date**, ou observe sur une vraie machine et marque
`observed`. Liste de depart (connaissance au 2026-10-16, a confirmer) :

| Hote | Fichier(s) pressentis | Format |
|---|---|---|
| Claude Desktop (config) | `~/.config/Claude/claude_desktop_config.json` (`mcpServers`) | JSON |
| Claude Desktop (extensions `.mcpb`, **observe le 2026-10-16 sur un build communautaire Linux** — pas de Claude Desktop officiel sous Linux, l'application Windows est re-empaquetee ; format a citer depuis la spec publique Anthropic, emplacement a comparer a macOS/Windows documentes) | `~/.config/Claude/Claude Extensions/<id>/manifest.json` (`server.mcp_config`, `tools[]`), `extensions-installations.json` (`extensions.<id>`), `Claude Extensions Settings/<id>.json` (`isEnabled`, valeurs `user_config`) | JSON |
| Claude Code | `~/.claude.json` (`mcpServers` global ; `projects.<chemin>.{mcpServers, enabledMcpjsonServers, disabledMcpjsonServers}` **observe**), `.mcp.json` dans les depots, `~/.claude/settings.json` | JSON |
| Cursor | `~/.cursor/mcp.json`, `<projet>/.cursor/mcp.json` | JSON |
| Windsurf | `~/.codeium/windsurf/mcp_config.json` | JSON |
| VS Code | `~/.config/Code/User/mcp.json`, `<projet>/.vscode/mcp.json`, `settings.json` (`mcp.servers`) | JSON avec commentaires |
| Cline / Roo / Kilo | `globalStorage/<ext>/settings/*mcp_settings.json` | JSON |
| Codex | `~/.codex/config.toml` (`[mcp_servers.*]`) | TOML |
| Gemini CLI | `~/.gemini/settings.json` (`mcpServers`) | JSON |
| OpenCode | `~/.config/opencode/opencode.json(c)`, `<projet>/opencode.json` (`mcp`) | JSON(C) |
| Continue | `~/.continue/config.yaml` (`mcpServers`) | YAML |
| Zed | `~/.config/zed/settings.json` (`context_servers`) | JSON |
| Goose | `~/.config/goose/config.yaml` (`extensions`) | YAML |
| Amp, LM Studio, JetBrains, Warp, 5ire, Kiro | a etablir | — |

Trois types de transport a reconnaitre : `stdio` (commande + arguments + env), `http`/
`streamable-http` et `sse` (URL + en-tetes). Les connecteurs distants de Claude.ai ne sont
pas sur le disque : hors inventaire, dit explicitement dans l'interface.

## 5. Regles du projet (au-dela de CONVENTIONS.md)

- **R1 — Lecture seule, sans exception.** MCProster lit ; il n'ecrit jamais dans un fichier
  d'hote, ne cree ni ne supprime de configuration, ne lance aucune commande d'hote. Les
  corrections sont affichees (fichier, ligne, commande a copier), jamais appliquees.
  Decision 1 et 5 du 2026-10-16.
- **R2 — Aucune sortie reseau.** Pas de registre npm/PyPI, pas de mise a jour, pas de
  telemetrie, meme sur demande. Un outil qui lit des cles d'API et parle au reseau ne doit
  pas exister. Decision 4. Exception bornee : la sonde (R4) lance un serveur configure par
  l'utilisateur ; c'est ce serveur qui peut, lui, parler au reseau (un `npx -y` telecharge)
  — l'ecran le dit avant.
- **R3 — Secrets masques partout, jamais ecrits.** Detection par nom (`KEY`, `TOKEN`,
  `SECRET`, `PASSWORD`, `CREDENTIAL`…) et par forme (`sk-`, `ghp_`, `xoxb-`, `AKIA`, longue
  chaine aleatoire) ; en cas de doute, masque. Affichage `sk-ant-…3f2a`, revelation une
  valeur a la fois par geste explicite, aucune valeur en clair dans un export, un cache, un
  journal. Decision 4.
- **R4 — Sonde sur demande seulement.** Par defaut l'inventaire est statique (fichiers,
  presence des commandes et paquets). Un bouton « Tester » (ou « Tout tester ») lance le
  serveur avec sa configuration, fait la poignee de main `initialize`, demande
  `tools/list`, l'arrete. Delai 10 s. Aucun outil n'est jamais appele. Le resultat est date
  et conserve pour l'affichage. L'ecran dit ce que ca implique avant le premier test.
  Decision 3.
- **R5 — Complet ou dit pourquoi pas.** Tout hote present sur la machine est lu ; un hote
  connu mais dont le fichier est illisible ou dans un format inattendu apparait avec
  l'erreur, pas en silence. Les six familles de faits (identite, declarations, version,
  etat, acces, outils exposes) sont les colonnes de l'inventaire. Decision 2.
- **R6 — Chaque hote est une regle declarative, sourcee.** `hosts/*.toml` : chemins, format,
  cle racine, correspondance des champs, commande d'hote pour retirer/ajouter si elle
  existe, source (documentation lue, date, citation) ou `observed`. Publie, contestable par
  issue. Meme schema de source que Cleanator, meme test « zero to-verify a la release ».
- **R7 — Un doublon est un fait, pas un jugement.** Deux declarations sont « le meme
  serveur » si meme paquet/binaire ou meme URL ; l'outil le dit avec les versions cote a
  cote, et laisse l'utilisateur decider laquelle garder.
- **R8 — Aucun fichier ecrit hors de son dossier.** Preferences (langue, theme, resultats
  dates des sondes) dans `~/.config/mcproster/`. Export de l'inventaire (JSON, Markdown)
  uniquement a la demande, a l'emplacement choisi, secrets masques (R3).
- **R9 — Anglais partout dans le code, les regles, les commits, la doc publique ; interface
  en anglais et francais.** Pilotage en francais a la racine avec en-tete anglais. Le seul
  nom cite est Alexandre Do.
- **R10 — Reutiliser Cleanator sans le fusionner.** i18n, preferences, paquets, style
  visuel, schema de source : repris. Pas de module dans Cleanator (autre question, autre
  outil).

## 6. Architecture envisagee

Meme recette que Cleanator, tranchee par continuite (decision 6) :

- `crates/core` — lecture des hotes (un lecteur par format : JSON, JSONC, TOML, YAML),
  modele commun `Server { name, transport, command|url, args, env (masque), version,
  declarations: [(host, scope, file, line)] }`, detection des doublons, des secrets, des
  acces larges, verification statique (commande dans `PATH`, paquet dans le cache npx/uv/
  pip, binaire executable), sonde MCP (`initialize` + `tools/list` sur stdio ; HTTP/SSE
  seulement si une URL locale repond — a trancher a l'etape 1), export JSON/Markdown.
- `crates/cli` — `mcproster hosts`, `mcproster list`, `mcproster findings`,
  `mcproster probe <server>`, `--json`.
- `crates/app` — iced 0.13 : une table « serveurs × hotes » (cases cochees ou vides), un
  panneau de constats a droite, detail d'un serveur (commande, env masque, declarations,
  outils exposes si sonde), bouton Tester, EN · FR, clair/sombre, preferences retenues.
- `hosts/*.toml` — les regles d'hotes (R6), `docs/hosts.md` leur schema.
- Paquets `.deb`, `.rpm`, AppImage par le meme `packaging/build.sh`.

## 7. Premieres taches (apres validation du BRIEF)

1. Etape 0 — Inventaire reel de la machine d'Alexandre : quels hotes, quels fichiers, quels
   formats, combien de serveurs, combien de doublons et de cles en clair (a la main, hors
   git, `00_cadrage/`). C'est ce qui corrige la table du §4 avant d'ecrire une regle.
2. Etape 1 — Regles d'hotes v0 avec sources lues ; lecteurs de format ; modele commun ;
   tests sur des fixtures : un faux `~/` riche (douze hotes, doublons a plusieurs versions,
   secrets, acces larges, fichiers illisibles) — la machine d'Alexandre n'a qu'un serveur
   (Eliot, `.mcpb`) et ne valide que ce lecteur-la.
3. Etape 2 — CLI complete et verifiee sur la vraie machine.
4. Etape 3 — Application iced (table, constats, detail, sonde sur demande).
5. Etape 4 — Paquets, depot `blackberrylabs/mcproster`, release 0.1.0 (push et release sur
   accord).
6. Etape 5 — README avec captures, comparatif verifie (outils existants : a chercher a
   l'etape 0 — il en existe peut-etre), CONTRIBUTING.

## 8. Hors perimetre

v1 : modifier une configuration (ajout, retrait, synchronisation) ; interroger un registre
(npm, PyPI, registre MCP officiel) ; appeler un outil d'un serveur ; connecteurs distants
Claude.ai ; Windows et macOS ; scan planifie ; telemetrie ; toute version payante.
Candidats v2 : copie corrigee d'un fichier de configuration ecrite a cote de l'original ;
« derniere version publiee » sur demande explicite ; suivi dans le temps (qu'est-ce qui a
change depuis le dernier inventaire).

## 9. Regime : cadrage avant code

Rien n'est code avant la validation de ce BRIEF ; l'etape 0 (inventaire reel) precede
l'ecriture de la premiere regle d'hote ; aucune mise en ligne avant que la conception soit
finie.

## 10. Questions tranchees (datees)

Tranchees le 2026-10-16 (conversation Cowork, entree JOURNAL du meme jour) :

1. Inventaire pur, pas de gestion (« on commence par l'inventaire pur, en sorte que cet
   inventaire soit vraiment complet »).
2. Complet = tous les hotes presents + six familles de faits ; connecteurs distants
   Claude.ai hors perimetre, dit a l'ecran.
3. Sonde sur demande (option C), delai 10 s, poignee de main seulement ; pas de registre.
4. Secrets masques partout, revelation unitaire, jamais ecrits ; aucune sortie reseau.
5. Constat precis + commande d'hote copiable ; pas de reecriture de fichier en v1.
6. Forme par continuite avec Cleanator ; nom **MCProster** (`mcproster`), libre sur
   crates.io et sans homonyme etabli (« scan » ecarte : signifie securite dans l'ecosysteme
   MCP — mcp-scan/Snyk, MCPScan/Ant Group, mcp-scanner/Cisco).

Tranchees a l'etape 0-2 : sonde HTTP/SSE → non (R2) ; projets → racines `--root` + les
projets connus de Claude Code ; outils existants → compares (§3).

## 11. Version 0.2 — « ce que vos serveurs MCP coutent en contexte » (cadrage du 2026-10-16)

### Origine

Lecture des douleurs de la communaute le 16/10 (`00_cadrage/2026-10-16_douleurs_communaute.md`) :
les serveurs MCP mangent la fenetre de contexte avant le premier mot — un utilisateur a
mesure 43 000 tokens sur 99 000 au demarrage pour 127 outils (A. Szablowski, 2026), au prix
d'un proxy de journalisation. MCProster a deja la donnee sous la main (`tools/list` de la
sonde) et ne la garde pas. La 0.2 la montre. Alexandre : « on part sur MCP roster 0.2 ».

### Decisions (une par une, 16/10)

- **D7 — Tester tout.** `mcproster probe --all` et un bouton dans l'app : tous les serveurs
  stdio actives, une commande par lancement distinct, **une seule confirmation qui liste les
  commandes** qui vont etre lancees (« ne plus demander » retenu comme en 0.1). Sans sonde,
  pas de cout : R4 inchangee.
- **D8 — Seuils absolus** : un serveur au-dessus de ~5 000 tokens, ou un hote dont la somme
  depasse ~20 000 (≈ 10 % d'une fenetre de 200 000) → constat orange ; en dessous, information.
  Chiffres ecrits et sources dans `docs/context-cost.md` ; mode relatif seulement si demande.
- **D9 — On garde des chiffres, pas des schemas** : par resultat de sonde, octets des schemas
  et estimation de tokens ; par outil, **nom et part en tokens** (un chiffre). Jamais le JSON
  des outils ni leurs descriptions sur disque : R3 et R8 intactes, `probes.json` reste petit.
- **D10 — Perimetre** : le cout en contexte + (a) commandes de retrait dans `findings` en
  CLI, (b) paquet present ou non dans les caches `npx`/`uv`, (d) captures d'ecran dans le
  README. (c) Kilo Code **reporte** : refondu en 2026, configuration encore mouvante.

### Ce que ca fait

- **Mesure** : pour chaque outil renvoye par `tools/list`, longueur en caracteres du JSON
  (`name`, `description`, `inputSchema`, et `outputSchema`/`annotations` s'ils existent) ;
  estimation de tokens = caracteres / 4, arrondie, toujours affichee avec « ~ ». Hypotheses
  documentees : chaque hote serialise a sa facon (±20 %) ; Claude Code peut **differer** le
  chargement des outils MCP (recherche d'outils) — la ligne de cet hote le dit.
- **Par serveur** : nombre d'outils, ~tokens, les trois outils les plus lourds. **Par hote** :
  somme des serveurs actives qu'il declare et qui ont ete sondes ; s'il en reste de non
  sondes, le total est « au moins ». Dans l'app : une colonne « ~tokens » dans la table, le
  detail des outils avec leur part, une ligne par hote au-dessus des constats.
- **Constat `context-cost`** (D8) avec l'action affichee : desactiver dans l'hote (`disabled`,
  `enabled = false`, reglages), ou, pour Claude Code, la recherche d'outils a la demande.
- **Reliquats** : (a) `remove_hint` dans `findings` CLI ; (b) constat d'information
  `package-not-cached` quand `npx -y <paquet>` / `uvx <paquet>` ne trouve pas le paquet dans
  `~/.npm/_npx/*/node_modules/<paquet>` ou le cache uv — « marchera, mais telechargera » ;
  (d) deux captures (clair, sombre) dans le README.
- **Ce qui ne change pas** : lecture seule, aucune sortie reseau, sonde sur demande, serveurs
  distants non sondes, `~/.config/mcproster/` seul dossier ecrit.

### Tests

Faux serveur avec des schemas lourds → estimation au-dessus du seuil → constat ; `probe --all`
sur le faux home (commandes dedoublonnees, confirmation, `--yes`) ; cache npx/uv simule dans
le faux home ; « aucun schema ni description d'outil dans `probes.json` » ; les tests 0.1
inchanges.

### Livraison

0.2.0 : CHANGELOG, paquets reconstruits, tag et release sur accord ; mise a jour de
`docs/hosts.md` si une regle bouge. Textes de presentation : decision d'Alexandre, pas le
jour de la sortie.
