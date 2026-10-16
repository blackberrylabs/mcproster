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
- [ ] Sonde stdio : `initialize` + `tools/list`, delai 10 s, aucun outil appele ; resultats dates dans `~/.config/mcproster/`
- [ ] Sonde HTTP/SSE : a trancher (question ouverte du BRIEF)
- [ ] Passage sur la machine d'Alexandre, sortie relue, defauts corriges

## Etape 3. Application iced

- [ ] Table serveurs × hotes, panneau de constats, detail d'un serveur, bouton Tester (avec l'avertissement avant le premier)
- [ ] EN · FR, clair/sombre, preferences retenues (repris de Cleanator)
- [ ] Validation visuelle par Alexandre

## Etape 4. Paquets, depot, release 0.1.0

- [ ] `packaging/build.sh` repris ; `.deb` installe et teste
- [ ] Depot `blackberrylabs/mcproster` — push sur accord
- [ ] Tag, release — sur accord

## Etape 5. README et presentation

- [ ] README avec captures, comparatif verifie, CONTRIBUTING, CHANGELOG
- [ ] Textes de presentation — plus tard, sur decision d'Alexandre
