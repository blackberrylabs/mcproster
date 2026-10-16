# JOURNAL.md — MCProster

> Author's working notes, in French. Public documentation lives in `README.md` and `docs/`.

Journal antichronologique des decisions. La plus recente en haut. Une entree = une date, un
titre, ce qui a ete decide, par qui, ce qui a ete ecarte, et la preuve (fichier, commit, run).

---

## 2026-10-16, nuit : passage reel de la 0.2

- `probe --all` chez Alexandre : la confirmation liste les deux lancements, `y`, les deux
  repondent. Apipriceindex 3 outils ~532 tokens (~177/outil), Eliot 22 outils ~2 370
  (~108/outil, descriptions en francais, courtes). Aucun des deux n'approche 5 000 : le
  seuil separe bien le normal (quelques centaines par outil) du lourd (les 26 outils / 33 Ko
  de l'article). Le constat doublon reste le seul.
- Defaut de lecture vu sur la sortie : « Claude Desktop » et « Claude Desktop (extensions,
  .mcpb) » sont deux regles mais une application ; le total reel etait ~2 900 et non deux
  lignes. Corrige : champ `application` dans les regles d'hote (les deux regles Claude
  Desktop le partagent), le rapport de cout somme par application, une ligne. Test ajuste,
  `docs/context-cost.md` et `docs/hosts.md` mis a jour.
- Paquets 0.2.0 reconstruits avec cette correction ; empreintes verifiees sur la machine.

## 2026-10-16, nuit : la 0.2 codee

- Coeur : chaque outil de `tools/list` est pese (caracteres du JSON / 4, « ~ »), par
  serveur et par hote (`cost.rs`) ; constat `context-cost` aux seuils D8 ; `actions.rs`
  sort du code de l'app pour servir aussi la CLI. `probes.json` ne garde plus les
  descriptions d'outils (D9) ; les sondes echouees sont comptees a part des non sondees.
- CLI : `probe --all` avec confirmation listant les commandes (`--yes` pour les scripts),
  `context`, actions sous chaque constat. En mode `--home`, les commandes sont cherchees
  dans le PATH de la machine courante (un backup monte n'a pas de PATH).
- App : *Test all* → panneau listant les lancements, sondes en file une par une, colonne
  ~tokens (orange au-dessus du seuil, ✗ si la sonde a echoue), bloc « coût par hôte » au-dessus
  des constats, KPI Findings incluant les constats de cout.
- Le faux home a maintenant des caches npx/uv simules ; le faux serveur a un mode
  `FAKE_HEAVY` (14 outils, > 5 000 tokens). 33 tests verts, clippy propre.
- Observation en sondant le faux home avec `--all` : le `npx` reel du cloud a telecharge et
  lance `@playwright/mcp` — la sonde fait exactement ce que l'hote ferait, c'est voulu, et
  c'est pour ca que la confirmation liste les commandes.
- README 0.2 avec deux captures d'un home de demonstration (serveurs Python qui repondent,
  un lourd, un doublon declare trois fois, un desactive, un secret, un distant).
  CHANGELOG 0.2.0, `docs/context-cost.md`.

## 2026-10-16, soir : cadrage de la 0.2

- Alexandre : « Quel est le prochain module le plus interessant ? » puis « regarde dans la
  communaute quelles sont les vraies douleurs ». Recherche web, synthese dans
  `00_cadrage/2026-10-16_douleurs_communaute.md` : (1) agents qui suppriment — incidents de
  50 a 370 Go documentes, aucun inspecteur lecture seule multi-hotes ; (2) serveurs MCP qui
  mangent le contexte — 43k/99k tokens mesures ; (3) fichiers d'instructions trop longs —
  limites 200 lignes / 32 Kio ; (4) secrets en clair — surtout dans les apps livrees, terrain
  occupe ; l'idee « secrets dans les traces » retrogradee.
- Decision d'Alexandre : **MCProster 0.2 d'abord** (cout en contexte), l'inventaire des
  permissions des agents comme troisieme outil ensuite.
- Quatre decisions une par une (BRIEF §11 : D7 a D10). Addendum ecrit, PLAN etape 6 ouverte.
  Attend la validation d'Alexandre avant le code.
- Regle de maison posee le meme soir dans `00_moi/CONVENTIONS.md` : outils separes, fondation
  commune, la tour de controle rassemble sans absorber.

## 2026-10-16, MCProster 0.1.0 publie

- Tag `v0.1.0` sur `main`, release GitHub avec les quatre fichiers. Verifie depuis le cloud :
  les quatre telecharges, `sha256sum -c` OK, `.deb` extrait → `mcproster 0.1.0`, inventaire
  du faux home identique. `CLAUDE.md` (regle d'identite git) present sur le tag et sur `main`.
- Reglages du depot : Issues et Pull requests seuls ouverts, Wikis / Discussions / Projects /
  Sponsorships fermes, suppression automatique des branches fusionnees. *About* pose par
  Alexandre (description, site = releases, topics).
- Deux logiciels publies le meme jour sous Blackberrylabs (Cleanator 0.1.1 le matin,
  MCProster 0.1.0 l'apres-midi). MCProster passe en veille active ; etape 5 (captures dans le
  README, textes de presentation) sur decision d'Alexandre, pas le jour de la sortie.

## 2026-10-16, depot en ligne

- Premier push refuse : « push declined due to email privacy restrictions ». Les 8 commits
  portaient l'identite globale du terminal (`Nimeria2026 <nimeria2026@outlook.fr>`) ;
  Cleanator avait une config **locale** noreply, MCProster pas encore. Meme incident que
  le 2026-10-16 sur Cleanator. Correction : config locale `Alexandre Do
  <277202463+Nimeria2026@users.noreply.github.com>`, `git filter-branch --env-filter` sur
  les 8 commits (dates et messages conserves), push OK. Regle ajoutee a `CLAUDE.md` :
  **config locale avant le premier commit**, a verifier par `git log --format='%an <%ae>'`.
- `blackberrylabs/mcproster` public, verifie depuis le cloud : README, LICENSE, regles,
  fixtures, fichiers de pilotage ; aucune adresse personnelle. Les faux secrets des
  fixtures (`sk-proj-abcdef…`) sont publics par construction : motif reconnaissable,
  invalide.
- Alexandre pose le *About* (description, site = releases, topics), Issues seules ouvertes.

## 2026-10-16, .deb installe ; pret pour le depot

- Alexandre : « Ok, tout fonctionne » — `.deb` installe, binaires et entree de menu en place.
- Note de version prete : `00_cadrage/release_v0.1.0_notes.md`, a coller dans la release.
- Reste, sur accord explicite : creation de `blackberrylabs/mcproster` (vide, sur GitHub),
  remote SSH comme Cleanator, push des 8 commits ; puis tag `v0.1.0`, push du tag, release
  avec les trois paquets et `SHA256SUMS` (fichiers dans `00_cadrage/dist/` ; `.rpm` et
  AppImage a deposer depuis le cloud au moment de la release).

## 2026-10-16, etape 4 : paquets

- Alexandre valide l'application telle quelle. Etape 3 close.
- Icone originale (une feuille d'appel : trois serveurs coches, le point orange sur celui
  declare deux fois), rendue en PNG par cairosvg (pas de rsvg dans le cloud). Entree
  `.desktop` EN/FR, categories Development;Utility.
- `packaging/build.sh` repris de Cleanator par substitution ; metadonnees `cargo-deb` et
  `generate-rpm` dans `crates/app/Cargo.toml` (un paquet, deux binaires :
  `mcproster-app` et `mcproster`). Premier echec : `LICENSE` n'existait que sur la machine
  d'Alexandre, pas dans l'arbre cloud — recree a l'identique.
- Paquets 0.1.0 construits : `.deb` 4,6 Mo (depend libc6 ≥ 2.39, libxkbcommon0,
  wayland | x11), `.rpm` 5,0 Mo, AppImage 7,5 Mo ; SHA256SUMS de la version seule.
- README complet, CHANGELOG 0.1.0, CONTRIBUTING. Le `.deb` est depose dans
  `00_cadrage/dist/` pour l'installation de test.

## 2026-10-16, etape 3 : l'application

- Maquette proposee en blocs ; Alexandre : « anglais par defaut » et « mets-toi dans la peau
  d'un developpeur ». Reponse honnete : les blocs ne tiennent pas a trente serveurs ; table
  dense + detail + filtres, boutons de copie (rien n'est selectionnable dans iced), menu
  d'hote. Valide : « Part sur la table, plus details, plus filtres. »
- `crates/app` (iced 0.13.1, repris de Cleanator sans fusion : settings, i18n par
  `locales/*.json`, presse-papiers, `xdg-open`). Anglais au demarrage quelle que soit la
  langue systeme — difference voulue avec Cleanator.
- Captures sous Xvfb sur le faux home : a 19 hotes configures, les puces debordaient et
  19 colonnes etaient illisibles → menu deroulant et colonnes par hote seulement jusqu'a 6
  (au-dela, une colonne « declared in »). Un fichier partage affichait l'identifiant
  `vscode` au lieu du nom → corrige dans le coeur (test ajoute). Chemins absolus
  raccourcis en `~` dans la table.
- Le clic sur une ligne et le detail n'ont pas pu etre captures (pas de `xdotool` dans le
  cloud) : a valider visuellement par Alexandre.
- Alexandre : « essaye de donner le meme visuel que Cleanator ». Rhabillage avec les briques
  de Cleanator : cartes KPI (Servers / Hosts with configuration / Findings / Secrets in
  clear, pastille verte ou orange), panneau a titre et sous-titre, pastilles de constats,
  palette verte `008300` / orange `eb6834` / gris `7a869a`, monospace pour identites,
  chemins et commandes, pied statut + EN·FR + theme, page `padding 16 / spacing 14`.
  Captures clair et sombre relues ; la barre de filtres debordait → champ elastique.
- Binaire `mcproster-app` (14 Mo) depose hors git dans `00_cadrage/bin/`.

## 2026-10-16, etape 2 close : sondes reelles

- `probe apipriceindex` : OK en 297 ms, protocole 2025-06-18, 3 outils (`search_models`,
  `get_price`, `estimate_cost`). `probe eliot` : OK en 2 ms, Eliot 0.1.0, 22 outils.
- Defaut d'affichage trouve par le passage reel : Apipriceindex renvoie une version
  **vide** dans `serverInfo`, affichee comme presente (« apipriceindex , protocol »).
  Corrige : vide = absente ; test ajoute (faux serveur avec `FAKE_VERSION=`).
- Retour pour Apipriceindex (pas MCProster) : `apipriceindex-mcp` ne declare pas sa
  version dans `initialize` — parametre `version` du `MCPServer` (mcp>=2). A noter dans
  ce projet-la.
- Etape 2 close. Prochaine : etape 3, application iced — maquette a valider avant le code.

## 2026-10-16, premier passage reel et sonde

- `mcproster list|findings|files` sur la vraie machine : 6 hotes avec fichier, 3
  declarations, 1 constat (le doublon volontaire `apipriceindex-mcp`, Claude Code + Claude
  Desktop). Claude Code ne connait que deux projets ici (MonQuiver.fr, Nimeria_poa), aucun
  `.mcp.json` dedans. `files` affichait « 0 declaration » pour le registre et les reglages
  des extensions : corrige (« versions and install dates », « enabled flag »).
- Sonde (`probe.rs`) : lance le serveur comme l'hote (commande, arguments, variables
  relues en clair dans le fichier au moment de la sonde, passees au processus enfant,
  jamais gardees), `initialize` → `notifications/initialized` → `tools/list`, delai global
  10 s, processus tue a la fin. Resultat : nom et version du serveur, version du
  protocole, noms d'outils (premiere ligne de description, 120 caracteres), erreur, debut
  de stderr en cas d'echec. Faux serveur Python pour les tests (reponse, delai depasse,
  commande absente, references `${VAR}` developpees pour l'enfant seulement).
- Decision prise et soumise a Alexandre : **pas de sonde HTTP/SSE en v0**, la regle R2
  (aucune sortie reseau) l'emporte sur la question ouverte du BRIEF.
- Resultats de sonde ecrits dans `~/.config/mcproster/probes.json`, seul fichier que
  MCProster ecrit (R8) ; `list` les rappelle (« probed 2026-10-16: 2 tool(s) »).

## 2026-10-16, etape 1 : le coeur, la CLI, le faux home

- Workspace Rust (`crates/core`, `crates/cli`), `#![forbid(unsafe_code)]`, regles d'hotes
  embarquees par `include_str!` et validees au chargement (une regle sans source est
  refusee — la validation a attrape un oubli dans la regle Continue des le premier essai).
- Coeur : `env` (home, variables, OS, PATH, racines — jamais `std::env` dans l'inventaire,
  les tests sont hermetiques), `read` (JSON, JSONC, TOML, YAML → un seul arbre ; `dig` avec
  `*` et `\.`), `secrets` (reference / secret / ordinaire ; masque `6…4`), `model`,
  `inventory` (resolution des emplacements, extraction, fusion des fichiers partages,
  metadonnees des extensions `.mcpb`, constats, regroupement par identite).
- Identite d'un serveur : URL, sinon paquet derriere `npx`/`uvx`/`pipx`/`bunx` (version
  retiree), sinon nom du binaire ; pour `node`/`python`, le script compte.
- Faux home de test : 19 hotes, 32 fichiers, `__HOME__` remplace a la copie. 26
  declarations, 19 constats attendus ; tests « rien n'est ecrit » et « aucun secret en
  clair dans toute la sortie JSON ».
- Deux defauts trouves par les tests avant toute execution reelle : la cle `amp.mcpServers`
  contient un point (ajout de `\.` dans les chemins de cles) ; la portee *project* « dans
  le fichier » de Claude Code etait resolue comme un chemin relatif a un projet.
- CLI : `hosts`, `list`, `findings`, `files`, `--json`, `--home`, `--root`. Chemins
  raccourcis en `~`, `| head` ne fait plus paniquer.
- Le pont de fichiers refuse d'ecrire `.claude.json`, `.mcp.json` et `.vscode/` meme dans
  un dossier de fixtures : ecrits par le shell a la place, empreintes verifiees.
- Binaire release compile dans le cloud (glibc 2.39 = Ubuntu 24.04), depose hors git dans
  `00_cadrage/bin/mcproster` pour le premier passage sur la vraie machine.

## 2026-10-16, etape 1 : regles d'hotes sourcees

- 19 hotes, 38 emplacements, ecrits en TOML (`hosts/`), chaque emplacement avec sa source :
  documentation editeur (URL, date, citation), code source quand la doc ne donne pas le
  chemin (Zed `paths.rs`, Cline `disk.ts`, Roo `globalFileNames.ts` + `McpHub.ts`), ou
  `observed` (Linux de Claude Desktop, registre des extensions, `settings.json` legacy de
  VS Code, `config.json` legacy de Continue). Rien de memoire : ce qui n'a pas pu etre lu
  est marque « derived, to confirm » dans la note.
- 6 hotes ecartes de la v0 avec la raison (`hosts/_not-read.toml`) : Kilo Code (reconstruit
  en 2026, nouvel emplacement non documente), JetBrains (XML interne), Warp (stockage
  interne), 5ire et Trae (non etablis), Aider (pas de MCP).
- Decouvertes consignees dans `docs/hosts.md` : `.mcp.json` lu par Claude Code **et**
  VS Code (un fichier, deux hotes → partage, pas doublon) ; six syntaxes de reference a
  l'environnement a ne jamais masquer ; serveurs desactives = declaration quand meme ;
  `trust`/`autoApprove`/`alwaysAllow` = constat.
- Schema d'une regle documente dans `docs/hosts.md` ; BRIEF §4 renvoie vers cette reference.
- A faire relire par Alexandre avant d'ecrire le lecteur Rust.

## 2026-10-16, etape 0 close : banc d'essai reel, depot git

- Trois commandes passees : `claude mcp add --scope user apipriceindex -- <binaire>` (binaire
  Claude Code embarque dans l'app, appele par son chemin), ajout de la meme cle dans
  `claude_desktop_config.json` (sauvegarde `.bak-2026-10-16` d'abord, JSON verifie), inventaire
  relance : **9 hotes, 3 declarations, 1 serveur declare deux fois, 0 secret** — exactement ce
  qui etait attendu.
- Observations pour les regles d'hotes :
  - `claude mcp add --scope user` ecrit dans `mcpServers` **a la racine** de `~/.claude.json`
    (pas sous `projects`) ; la cle n'existe pas tant qu'aucun serveur n'est declare.
  - `claude_desktop_config.json` fait 9 Ko : il porte d'autres reglages que `mcpServers`,
    MCProster ne lit que cette cle et ne commente pas le reste.
  - Apres redemarrage du PC, Claude Desktop (build communautaire) montre le nouveau serveur
    dans Reglages › Connecteurs avec ses trois outils (`search_models`, `get_price`,
    `estimate_cost`), permission par defaut « necessite une approbation ». Rien n'est su
    sur une prise en compte a chaud : Claude avait d'abord cru a un rechargement sans
    redemarrage, Alexandre a corrige (le PC avait ete redemarre).
  - L'extension Eliot a pour identifiant `local.mcpb.alexandre-do.eliot` : prefixe `local.mcpb.`
    pour une extension installee depuis un fichier, pas depuis le repertoire. Utile pour
    distinguer « extension du catalogue » et « extension locale » dans l'inventaire.
- Depot git initialise (`main`, commit 1556ed2), identite globale deja correcte. Pas de
  depot distant tant qu'il n'y a pas de code.
- Etape 0 close dans PLAN.md. Suite : etape 1 (regles d'hotes sourcees, lecteurs, fixtures).

## 2026-10-16, serveur Apipriceindex installe ; Claude Desktop communautaire

- Alexandre propose d'installer son propre serveur MCP (`apipriceindex-mcp` 0.1.1, PyPI,
  registre MCP officiel, 3 outils ; depot `Apipriceindex/mcp-public`, lu par Claude) : il
  devient utilisateur a deux mecanismes (Eliot en `.mcpb`, Apipriceindex en `mcpServers`)
  et fabrique un doublon volontaire Claude Code / Claude Desktop pour l'inventaire.
  `uv tool install` fait ; `claude` absent du PATH → binaire Claude Code embarque dans
  l'app (`~/.config/Claude/claude-code/2.1.284/claude`), appele par son chemin.
- Alexandre : « j'utilise Claude Desktop communautaire » (pas d'officiel sous Linux ; l'autre
  PC a l'officiel). Consequence consignee au BRIEF §4 : tout ce qui est observe dans
  `~/.config/Claude/` l'est sur un build communautaire ; la regle d'hote citera la spec
  `.mcpb` d'Anthropic et comparera l'emplacement aux plateformes documentees. La plupart des
  utilisateurs Linux seront dans ce cas.
- Lecture de la reponse d'Alexandre a la question « on continue ? » : oui — un auteur de
  serveurs MCP qui veut voir ou ils sont declares est exactement le public.

## 2026-10-16, etape 0 : un serveur, deux mecanismes, trois gestionnaires

- Inventaire reel : 9 hotes avec fichier, 0 serveur dans `mcpServers` partout ; le seul
  serveur local d'Alexandre, Eliot, est une **extension `.mcpb` de Claude Desktop**
  (manifeste, registre `extensions-installations.json`, reglages). Le BRIEF rev. 1 ignorait
  ce mecanisme : trouve grace au relevé de structure (cles seulement, jamais de valeurs).
  Alexandre : les connecteurs Qonto/Gmail/Canva… vus en session « c'est mon frere » (compte
  partage) — et ce sont des connecteurs distants, hors disque.
- Veille : mcpm.sh (1 014★), mcp-linker (342★, Tauri, AGPL + commercial), AreteDriver/
  mcp-manager (CLI Python, health-check `tools/list`), mcp-sync — des gestionnaires qui
  ecrivent et synchronisent ; aucun inspecteur en lecture seule, aucune app de bureau en un
  binaire. Positionnement MCProster : « l'inspecteur, pas le gestionnaire ».
- BRIEF revision 2 et `00_cadrage/2026-10-16_inventaire_analyse.md`. Question posee a
  Alexandre avant l'etape 1 : public plus etroit que Cleanator (il faut plusieurs hotes
  avec des serveurs ; lui n'en a qu'un) — on continue ?

## 2026-10-16, BRIEF valide ; script d'inventaire pret

- Alexandre : « Je valide. » (BRIEF rev. 1 dans son ensemble). Etape 0 ouverte.
- Fait par Claude : `00_cadrage/inventaire_mcp.py`, lecture seule, Python 3.12 sans
  dependance (tomllib ; PyYAML optionnel pour Continue/Goose, signale s'il manque). 24
  emplacements d'hotes (BRIEF §4), lecteurs JSON/JSONC/TOML/YAML, par-projet via glob sous
  `~/Bureau` et `~/Projets` (hors node_modules, .git, target, venvs). Masque toute valeur
  au nom ou a la forme de secret (6 premiers + 4 derniers), ne masque pas les references
  `${VAR}` (bonne pratique). Doublons par paquet **sans sa version**, versions cote a cote ;
  constats : secrets en clair, acces larges (`/`, `/home`, `~`), versions divergentes,
  fichiers illisibles. Teste sur un faux home (5 hotes, 7 serveurs, 1 doublon a 3 versions,
  2 secrets, 1 acces large). Sortie : `00_cadrage/AAAA-MM-JJ_inventaire_mcp.md`, hors git.

## 2026-10-16, naissance du projet : six decisions et un BRIEF

- Alexandre, apres la release 0.1.1 de Cleanator : « on va pouvoir s'attaquer a un nouveau
  logiciel. Et je pense que l'inventaire MCP serait vraiment pas mal. » Idee consignee le
  matin meme dans `00_moi/IDEES.md` (outils separes sous Blackberrylabs).
- Six questions posees une par une par Claude, six reponses d'Alexandre :
  1. **Inventaire pur** (pas de gestion) — « en sorte que cet inventaire soit vraiment
     complet ».
  2. **Complet** = tous les hotes presents sur la machine + six familles de faits (identite,
     declarations, version, etat, acces, outils exposes) ; connecteurs distants Claude.ai
     hors perimetre, dit a l'ecran. « Pour moi, la, c'est complet. »
  3. **Sonde sur demande** (option C : statique par defaut, bouton Tester, poignee de main
     seulement, 10 s) ; pas de registre npm/PyPI en v1.
  4. **Secrets masques partout**, revelation unitaire, jamais ecrits ; **aucune sortie
     reseau** de l'outil.
  5. **Constat precis + commande d'hote copiable**, pas de reecriture de fichier en v1
     (copie corrigee en v2).
  6. **Forme par continuite** avec Cleanator (Rust, iced, CLI, en/fr, MIT, Blackberrylabs).
     **Nom : MCProster** (`mcproster`) — libre sur crates.io, un seul homonyme GitHub sans
     rapport. Ecartes : `mcpscope` et `mcpedia` (10–15 petits homonymes), `census`/`atlas`
     (collision avec des serveurs MCP connus), et « scan MCP », propose par Alexandre, parce
     que « scan » signifie securite dans l'ecosysteme (mcp-scan/Snyk, MCPScan/Ant Group,
     mcp-scanner/Cisco, 485 depots) — « MCP roster me va ».
- Dossier `Bureau/Logiciels/Mcproster/` cree par Alexandre a cote de Cleanator et Eliot ;
  acces accorde a Claude. BRIEF revision 1 depose ; a valider dans son ensemble avant
  l'etape 0 (inventaire reel de la machine).
