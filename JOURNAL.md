# JOURNAL.md — MCProster

> Author's working notes, in French. Public documentation lives in `README.md` and `docs/`.

Journal antichronologique des decisions. La plus recente en haut. Une entree = une date, un
titre, ce qui a ete decide, par qui, ce qui a ete ecarte, et la preuve (fichier, commit, run).

---

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
