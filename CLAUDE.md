# CLAUDE.md — MCProster

> Author's working notes, in French. Public documentation lives in `README.md` and `docs/`.

Point d'entree pour reprendre le travail sans contexte. Lire ensuite `BRIEF.md` (cadrage),
`PLAN.md` (avancement), `JOURNAL.md` (decisions recentes).

## Ce qu'est ce projet

MCProster fait l'appel des serveurs MCP d'une machine Linux : quel serveur est declare dans
quel assistant (Claude Desktop, Claude Code, Cursor, Codex, OpenCode, Continue, VS Code…),
avec quelle version, et ce qui cloche (doublons, serveurs morts, cles en clair, acces disque
larges). Il regarde et ne touche pas : lecture seule, aucune sortie reseau, secrets masques.
Second logiciel de Blackberrylabs apres Cleanator ; meme public, meme methode, meme recette
technique (Rust, iced, CLI, en/fr, MIT).

## Etat au 2026-10-16

BRIEF revision 1 ecrit, six decisions tranchees (voir JOURNAL). **Pas encore valide dans son
ensemble, pas une ligne de code.** Prochaine action : validation du BRIEF par Alexandre,
puis etape 0 (inventaire reel de sa machine, hors git, dans `00_cadrage/`).

## Regles de travail (reprises de Cleanator)

- Cadrage ecrit et valide avant le code ; questions une par une.
- Commits locaux sans validation ; `git push`, suppressions, release : accord explicite.
- Toute commande donnee a Alexandre est complete, copiable, avec chemin absolu
  (`cd /home/nushoko/Bureau/Logiciels/Mcproster && …`).
- Verrous git laisses par le shell de Cowork : `find .git -name '*.lock' -delete` avant un
  commit. `.gitignore` s'edite sur place, jamais recopie depuis le cloud.
- `00_cadrage/` est hors git : inventaire personnel, sorties, messages de commit.
- Claude compile et teste dans le cloud, copie les fichiers sur le PC (sommes verifiees).

## Arborescence prevue

`BRIEF.md`, `PLAN.md`, `JOURNAL.md`, `CLAUDE.md` (pilotage, francais) ; `hosts/` (regles
d'hotes, TOML, anglais) et `docs/hosts.md` ; `Cargo.toml` racine, `crates/core`,
`crates/cli`, `crates/app` ; `locales/`, `assets/`, `packaging/` ; `README.md`, `LICENSE`,
`CHANGELOG.md`, `CONTRIBUTING.md` (anglais) ; `00_cadrage/` hors git.

## Identite git (incident du 2026-10-16, deux fois)

Avant le **premier commit** d'un depot : `git config user.name "Alexandre Do"` et
`git config user.email "277202463+Nimeria2026@users.noreply.github.com"` en config locale ;
verifier par `git log --format='%an <%ae>'` avant tout push. La config globale du terminal
d'Alexandre est une autre identite et GitHub refuse le push (email privacy).
