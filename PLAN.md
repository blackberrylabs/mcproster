# PLAN.md — MCProster

> Author's working notes, in French. Public documentation lives in `README.md` and `docs/`.

Plan vivant, source de verite de l'avancement. Une etape = un resultat visible. Les cases se
cochent avec la date ; ce qui est ecarte reste, barre, avec la raison.

## Etape 0. Cadrage et inventaire reel (ouverte le 2026-10-16)

Resultat visible attendu : BRIEF valide dans son ensemble par Alexandre ; inventaire a la
main de la machine d'Alexandre (quels hotes, quels fichiers, quels formats, combien de
serveurs, de doublons, de cles en clair) dans `00_cadrage/`, hors git ; table du BRIEF §4
corrigee par ce qui a ete vu ; outils existants cherches et compares.

- [x] Six decisions tranchees une par une (inventaire pur ; perimetre complet ; sonde sur
      demande ; secrets masques, zero reseau ; constat + commande d'hote, pas de reecriture ;
      forme par continuite, nom MCProster) — 2026-10-16
- [x] BRIEF revision 1 ecrit — 2026-10-16
- [x] BRIEF valide dans son ensemble par Alexandre (« Je valide ») — 2026-10-16
- [x] Inventaire reel de la machine (`inventaire_mcp.py` + `structure_mcp.py`, deux
      passes) : 9 hotes avec fichier, **1 serveur** (Eliot, extension `.mcpb`) ; Claude
      Desktop a deux mecanismes, decouvert ici — 2026-10-16
- [x] Outils existants : mcpm.sh, mcp-linker, AreteDriver/mcp-manager, mcp-sync — tous des
      gestionnaires qui ecrivent ; aucun inspecteur en lecture seule (tableau dans
      `00_cadrage/2026-10-16_inventaire_analyse.md`) — 2026-10-16
- [x] BRIEF revision 2 (positionnement, Claude Desktop ×2, tests sur faux home) — 2026-10-16
- [x] Decision d'Alexandre : on continue — il installe son propre serveur (`apipriceindex-mcp`)
      pour faire un banc d'essai reel : Eliot (`.mcpb`) + Apipriceindex declare dans Claude Code
      et Claude Desktop ; inventaire relance : **3 declarations, 1 doublon**, 0 secret — 2026-10-16
- [x] Depot git local initialise (`main`, commit 1556ed2 « Project skeleton »), `.gitignore`,
      LICENSE MIT, README minimal — 2026-10-16

**Etape 0 close le 2026-10-16.**

## Etape 1. Regles d'hotes v0 et coeur

Resultat visible attendu : `mcproster hosts` liste les hotes connus avec leur source ;
`mcproster list` sur un faux home de test (fixtures : dix hotes, doublons, cles) donne la
feuille d'appel ; tests verts ; zero source `to-verify`.

- [x] `hosts/*.toml` : 19 hotes lus (38 emplacements), 6 hotes ecartes avec la raison
      (`_not-read.toml`) ; chaque emplacement cite sa source (doc, code source ou observe),
      la date et la citation ; `docs/hosts.md` genere — 2026-10-16
- [x] Relecture des regles par Alexandre : « rien ne me choque » — 2026-10-16
- [x] Lecteurs : JSON, JSONC (commentaires, virgules finales), TOML, YAML → un arbre commun, puis `Declaration` — 2026-10-16
- [x] Portees : user, project, system ; racines de projet = `--root` + les cles `projects` de `~/.claude.json` (reponse a la question ouverte du BRIEF) — 2026-10-16
- [x] Doublons (meme paquet, URL ou binaire ; `node <script>` compte le script), versions cote a cote ; fichier partage `.mcp.json` = une declaration pour deux hotes — 2026-10-16
- [x] Secrets : detection par nom et par forme, references `${…}` jamais masquees, test « aucune valeur en clair dans toute la sortie » — 2026-10-16
- [x] Verification statique : commande dans `PATH` ou chemin existant (`${__dirname}` resolu) ; cache npx/uv non lu (v0.2) — 2026-10-16
- [x] Acces larges : arguments-chemins, constat sur `/`, `/home`, `/Users`, `~`, racine Windows — 2026-10-16
- [ ] Commandes d'hote pour retirer, quand elles existent (`remove_hint` dans la regle : Claude Code, Codex), a afficher dans `findings` (v0.2)

Tests : 12 unitaires + 8 d'integration sur le faux home (`crates/core/tests/fixtures/home`,
19 hotes, 26 declarations, 19 constats attendus, « rien n'est ecrit », « aucun secret en clair »).

## Etape 2. CLI complete et verification sur la vraie machine

- [x] `mcproster hosts | list | findings | files`, `--json`, `--home <dir>`, `--root <dir>` (sonde : etape suivante) — 2026-10-16
- [x] Sonde stdio : `mcproster probe <serveur>`, `initialize` + `tools/list`, delai 10 s (`--timeout`), aucun outil appele, processus arrete ; resultats dates dans `~/.config/mcproster/probes.json` (noms, versions, outils — jamais une valeur de config), repris dans `list` — 2026-10-16
- [x] Sonde HTTP/SSE : **non, en v0** — R2 « aucune sortie reseau » tranche ; `probe` repond « remote server, not probed ». A revoir si Alexandre le souhaite — 2026-10-16
- [x] Premier passage sur la machine d'Alexandre : 6 hotes avec fichier, 3 declarations, le doublon volontaire ; `files` corrige (metadonnees ≠ declarations). Sonde reelle : Apipriceindex OK (3 outils, 297 ms, version absente dans
      `serverInfo` → retour pour le projet Apipriceindex), Eliot OK (22 outils, 2 ms, 0.1.0) — 2026-10-16

**Etape 2 close le 2026-10-16** (reste pour v0.2 : `remove_hint` dans `findings`).

## Etape 3. Application iced

- [x] Maquette discutee avec Alexandre (blocs → table dense + detail + filtres, « mets-toi dans la peau d'un developpeur ») — 2026-10-16
- [x] Table serveurs × hotes (colonnes par hote jusqu'a 6 hotes configures, sinon une colonne « declared in »), filtre texte, menu d'hote, bascules « constats seulement » / « voir desactives et anciens » ; detail d'un serveur (declarations, fichier:ligne, Open / Copy path / Copy command, variables masquees) ; panneau des constats avec l'action possible affichee (jamais executee), clic → selection ; bouton Test avec confirmation avant le premier (« ne plus demander » retenu), serveurs distants non sondes — 2026-10-16
- [x] EN · FR (**anglais par defaut** quelle que soit la langue systeme), clair/sombre, preferences dans `~/.config/mcproster/settings.json` — 2026-10-16
- [x] Validation visuelle par Alexandre : « je garderai tel quel, c'est tres bien » — 2026-10-16

**Etape 3 close le 2026-10-16.**

## Etape 4. Paquets, depot, release 0.1.0

- [x] `packaging/build.sh` repris ; icone originale (feuille d'appel, point orange), `.desktop`, metadonnees deb/rpm ; `.deb` 4,6 Mo, `.rpm` 5,0 Mo, AppImage 7,5 Mo construits dans le cloud, tests release verts — 2026-10-16
- [x] `.deb` installe et teste sur la machine d'Alexandre : « tout fonctionne » — 2026-10-16
- [x] Depot `blackberrylabs/mcproster` cree par Alexandre, 8 commits pousses apres reecriture de l'identite (noreply) — 2026-10-16
- [x] Tag `v0.1.0` pousse, release publiee avec `.deb`, `.rpm`, AppImage et `SHA256SUMS` ; verifiee depuis le cloud (4 fichiers telecharges, empreintes OK, binaire 0.1.0 fonctionnel) — 2026-10-16

**Etape 4 close le 2026-10-16. MCProster 0.1.0 est publie.**

## Etape 5. README et presentation

- [x] README complet (promesses, hotes, install, CLI, app, lecture des constats), CHANGELOG 0.1.0, CONTRIBUTING (ajouter un hote = un TOML avec sources) — 2026-10-16
- [ ] Captures d'ecran dans le README (apres installation chez Alexandre)
- [ ] Textes de presentation — plus tard, sur decision d'Alexandre

## Etape 6. Version 0.2 — cout en contexte (BRIEF §11, cadre le 2026-10-16)

- [x] Lecture des douleurs de la communaute ; classement ; decision d'Alexandre : 0.2 — 2026-10-16
- [x] Quatre decisions (D7 Tester tout, D8 seuils absolus, D9 chiffres pas schemas, D10 perimetre) — 2026-10-16
- [x] Validation de l'addendum BRIEF §11 par Alexandre : « Je valide, tu peux y aller » — 2026-10-16
- [x] Coeur : `probe.rs` mesure chaque outil (caracteres du JSON / 4), `cost.rs` (par serveur avec les 3 plus lourds, par hote avec non sondes / echecs / distants / Claude Code borne haute, constat `context-cost`), `actions.rs` (actions affichees, partagees CLI/app) ; `probes.json` : noms et nombres, descriptions plus ecrites — 2026-10-16
- [x] `probe --all` (une confirmation listant les commandes, `--yes`), `mcproster context` ; app : bouton *Test all*, panneau de confirmation listant les lancements, sondes en file, colonne ~tokens, bloc « coût par hôte » — 2026-10-16
- [x] Reliquats : actions sous chaque constat en CLI ; constat `package-not-cached` (npx `~/.npm/_npx`, uv `archive-v0`, pipx) — 2026-10-16
- [x] `docs/context-cost.md` (methode, seuils D8, ce qui est garde D9, caches, limites) — 2026-10-16
- [x] Tests : faux serveur lourd (14 outils, > seuil), rapport de cout sur le faux home avec sondes synthetiques (seuils, non sondes, echecs, borne haute), caches npx/uv simules, « aucune description dans probes.json » — 33 tests verts — 2026-10-16
- [x] Passage reel : `probe --all` avec confirmation, Apipriceindex ~532 tokens (3 outils), Eliot ~2 370 (22 outils) ; seuils confirmes plausibles ; defaut vu et corrige : cout regroupe par **application** (Claude Desktop config + extensions = une ligne) — 2026-10-16
- [x] Captures README (clair, sombre, home de demonstration), CHANGELOG 0.2.0, README 0.2 — 2026-10-16
- [x] Paquets 0.2.0 construits et verifies ; app validee par Alexandre (« c'est bon pour moi ») ; note de version prete — 2026-10-16
- [x] Tag `v0.2.0` pousse, release publiee ; verifiee depuis le cloud : quatre fichiers, `sha256sum -c` OK, `.deb` extrait → `mcproster 0.2.0`, rapport de cout identique — 2026-10-16

**Etape 6 close le 2026-10-16. MCProster 0.2.0 est publie.** Reste pour plus tard : Kilo Code, textes de presentation (decision d'Alexandre), `.rpm` a tester sur Fedora.

## Etape 7. Version 0.2.1 — migration sur `hostrules`

- [x] Coeur commun extrait dans `blackberrylabs/hostrules` v0.1.0 ; MCProster migre, sorties identiques — 2026-10-16
- [x] Passage reel sur la machine d'Alexandre : `list`, `findings`, `context` identiques au 0.2.0, hint `claude mcp remove` en place — 2026-10-16
- [x] Commit 11 (6e787ff) — 2026-10-16
- [x] Interface depuis `bbl-ui` 0.1.0, zero pixel de difference (commit 13) — 2026-10-16
- [x] `mcproster report` (`bbl-report/1`), `--select` dans l'app, mapping des gravites dans le CHANGELOG — 2026-10-16
- [ ] Paquets 0.2.1, release — sur accord ; quand la tour lira les rapports
