## [1.4.1](https://github.com/jwallace145/rusty-chess/compare/v1.4.0...v1.4.1) (2026-01-31)


### Bug Fixes

* **releases:** Fix ghost reference in PR template ([#10](https://github.com/jwallace145/rusty-chess/issues/10)) ([bc95bd0](https://github.com/jwallace145/rusty-chess/commit/bc95bd08798b9b1f4b2e6fd1968436c9d97a361c))

# [1.4.0](https://github.com/jwallace145/rusty-chess/compare/v1.3.0...v1.4.0) (2026-01-31)


### Features

* **releases:** Upload binaries necessary for distribution with each release ([#9](https://github.com/jwallace145/rusty-chess/issues/9)) ([5256c27](https://github.com/jwallace145/rusty-chess/commit/5256c277a30b7255bf37c9906ba897fcc6d3ce97))

# [1.3.0](https://github.com/jwallace145/rusty-chess/compare/v1.2.0...v1.3.0) (2026-01-31)


### Features

* **releases:** Update release workflow to bump Cargo version ([#8](https://github.com/jwallace145/rusty-chess/issues/8)) ([a25a7ab](https://github.com/jwallace145/rusty-chess/commit/a25a7abd857f4dd431059e1200ae65ee3a7702ed))

# [1.2.0](https://github.com/jwallace145/rusty-chess/compare/v1.1.0...v1.2.0) (2026-01-31)


### Features

* **engine:** Allow player to optionally select engine opening book ([#5](https://github.com/jwallace145/rusty-chess/issues/5)) ([66566e7](https://github.com/jwallace145/rusty-chess/commit/66566e733b619276d8724f57508dec3627490034))

# [1.1.0](https://github.com/jwallace145/rusty-chess/compare/v1.0.0...v1.1.0) (2026-01-31)


### Features

* update release automation bot ([73f0980](https://github.com/jwallace145/rusty-chess/commit/73f09801b0d1bd10725ded3e47b47ee47ddde953))

# 1.0.0 (2026-01-31)


### Bug Fixes

* pawns promote after complete advance ([8552896](https://github.com/jwallace145/rusty-chess/commit/8552896c66d3c7fcbaae5ccbf2e067f021eb378b))
* update failing minimax unit tests ([e21717c](https://github.com/jwallace145/rusty-chess/commit/e21717c60f229a732de8bcdf55a7ae74a8afbeac))


### Features

* add basic threat evaluator ([13cf439](https://github.com/jwallace145/rusty-chess/commit/13cf439ae2569f64cfbc75ba2af04e330c9d8776))
* add benchmark for profiling ([1b361d1](https://github.com/jwallace145/rusty-chess/commit/1b361d12593f449d4c8d2d317e188cd450de1f47))
* add bit board ([2af0642](https://github.com/jwallace145/rusty-chess/commit/2af06422d5641163b0192763a5f2d6911ae6886b))
* add bit board refactor ([a19818f](https://github.com/jwallace145/rusty-chess/commit/a19818f6e6109fd60fde0b11816de7c7913e5a38))
* add board evaluation cli tool ([523fa26](https://github.com/jwallace145/rusty-chess/commit/523fa26057e6a665fb9c9a495e8a6d83965e05fa))
* add board utils module ([e357c9b](https://github.com/jwallace145/rusty-chess/commit/e357c9b24f0658f74856737e4a7ec333ce604f2a))
* add changelog automation ([62f04dc](https://github.com/jwallace145/rusty-chess/commit/62f04dc51dde5c6273f9680c90594bbcd877d8a5))
* add changelog automation (attempt 2) ([c404d24](https://github.com/jwallace145/rusty-chess/commit/c404d24a2244609c4233018ea6c6e0711f84c63c))
* add chess board ([d83db2a](https://github.com/jwallace145/rusty-chess/commit/d83db2a5a627a7e6f65fbb03da1558e677cd9991))
* add chess board evaluation ([bc37286](https://github.com/jwallace145/rusty-chess/commit/bc37286ecaa899715c02d083ba7adf8f7fb926ff))
* add cli game demo ([2937864](https://github.com/jwallace145/rusty-chess/commit/29378646d68ad358feecfe80ef82902135c43d2b))
* add enemy attack pressure to king safety evaluation ([6fbc50d](https://github.com/jwallace145/rusty-chess/commit/6fbc50d38f97e7c57d693b1080bb026dcc15f63d))
* add fen board ouptut to ai game ([ce85ce0](https://github.com/jwallace145/rusty-chess/commit/ce85ce00ee814fc47b0cb8c9ef226dc22fdac3ee))
* add fork evaluator ([fb332f6](https://github.com/jwallace145/rusty-chess/commit/fb332f699de40cf38cf68ad57a2d4cc421364624))
* add game metrics output file ([0789c04](https://github.com/jwallace145/rusty-chess/commit/0789c04176f12b9ddc7fea80e2bb7f50883e602e))
* add iterative deepening ([5524ab0](https://github.com/jwallace145/rusty-chess/commit/5524ab04e2df5c36bf17fc2266eb13b3f64a8f0f))
* add king endgame position table ([0c66cc8](https://github.com/jwallace145/rusty-chess/commit/0c66cc8b01b1f5344c379d2569e019c4e9e10478))
* add line pressure evaluator ([4dd32f8](https://github.com/jwallace145/rusty-chess/commit/4dd32f80714e643af463695825e53b5a481fdec6))
* add logic for castling, en passant, and promotions ([5baa27a](https://github.com/jwallace145/rusty-chess/commit/5baa27a3eb1fd57bfcfbd6bd00a85abc74b1a047))
* add magic numbers ([e0c56d1](https://github.com/jwallace145/rusty-chess/commit/e0c56d124f2d78fdf0aba6d376e079b71c520b90))
* add minimax algorithm to find next best move ([fee63da](https://github.com/jwallace145/rusty-chess/commit/fee63da31c12efafb02d28b50e1ebd61cc2742d3))
* add mobility evaluator ([49147b5](https://github.com/jwallace145/rusty-chess/commit/49147b5c537dcf4ef7555d17c2c5e7070c44594e))
* add more tactical evaluation ([d4a6dfd](https://github.com/jwallace145/rusty-chess/commit/d4a6dfd6f851a61f4b2f7500d45882207d71b200))
* add most valuable victim and least valuable attacker move logic ([d7c8695](https://github.com/jwallace145/rusty-chess/commit/d7c8695ccb94d10d1a7276753f11df78f661d725))
* add options for displaying performance insights ([1538a47](https://github.com/jwallace145/rusty-chess/commit/1538a479f1adedbac2199bb61cae3aafc163c038))
* add pawn structure evaluator ([de6e170](https://github.com/jwallace145/rusty-chess/commit/de6e170fe25edb632d24fb93c461bb6ac76196c6))
* add performance improvements ([32b9c46](https://github.com/jwallace145/rusty-chess/commit/32b9c46ffef13a0d4c2001221c07361587ee0af4))
* add play against ai cli game ([fb2262c](https://github.com/jwallace145/rusty-chess/commit/fb2262cd099770428dc0c23c70fa7e190d796062))
* add positional evaluation to board state evaluation logic ([1a73aee](https://github.com/jwallace145/rusty-chess/commit/1a73aee08d4058830215c390415c7b9d68c92782))
* add pre-commit hook ([e4afb40](https://github.com/jwallace145/rusty-chess/commit/e4afb4047629a728055b79ea53f989513b502b12))
* add pull request and issue templates ([0b2dd07](https://github.com/jwallace145/rusty-chess/commit/0b2dd07d530102c29506c20ec2cfebd20124b91d))
* add quiescence search ([ce69353](https://github.com/jwallace145/rusty-chess/commit/ce69353cb4d0b35e8c9046e5b6ec0f51aadca104))
* add resign and quit to player options ([e9b3817](https://github.com/jwallace145/rusty-chess/commit/e9b3817cea0be8b40edae75071bb50350f670b93))
* add search history and repetition detection ([192e912](https://github.com/jwallace145/rusty-chess/commit/192e91286a7feb036703194eca62c748079f384b))
* add simple opening book ([4766c16](https://github.com/jwallace145/rusty-chess/commit/4766c16e439192e44b3c180b1adb6737c5fce0f6))
* add static exchange evaluation to threat evaluator ([c3a2087](https://github.com/jwallace145/rusty-chess/commit/c3a2087ccda3f254742ee9f4fc85cec89d5007a5))
* add transposition table size in bytes to console output ([6b87f5f](https://github.com/jwallace145/rusty-chess/commit/6b87f5fa30c7c7f5e45a42fe2cd1358a4391f822))
* add unit tests for chess board ([9837a4e](https://github.com/jwallace145/rusty-chess/commit/9837a4e0896c28291d1500e0a37673382e295e85))
* add zobrist hashing and transposition table ([235f3f2](https://github.com/jwallace145/rusty-chess/commit/235f3f2352de5fb9ff925bb3e028cd525f14ae52))
* bitboard chess board implementation ([16973aa](https://github.com/jwallace145/rusty-chess/commit/16973aa4f8cbc03b1e962994bd145db4b0352159))
* generate magic numbers ([e6ea9ae](https://github.com/jwallace145/rusty-chess/commit/e6ea9ae16adeb981bd3ba6718e33b1efe6007cb7))
* improve move ordering during search ([684082f](https://github.com/jwallace145/rusty-chess/commit/684082fdb9a42e7ef46d242718f67c5ceeedade6))
* organize attacks database ([b73e207](https://github.com/jwallace145/rusty-chess/commit/b73e20761ec53a4636c91bc017af4a930a130480))
* organize move generation logic ([5bb8b56](https://github.com/jwallace145/rusty-chess/commit/5bb8b56aaecc468a8a22353f80c3ff6c0eaeedff))
* organize move generation logic ([2f2ffb0](https://github.com/jwallace145/rusty-chess/commit/2f2ffb04311635186f6bd455e28e83ded60856b4))
* refactor evaluators ([617fd77](https://github.com/jwallace145/rusty-chess/commit/617fd77be1d2e61fd0dbe4fd1d8437b909d0fd48))
* tone down king safety weighting ([ab741a5](https://github.com/jwallace145/rusty-chess/commit/ab741a580ddc010029a1c60dc897bd0dc1c3055a))
* update generate legal moves benchmark ([e837363](https://github.com/jwallace145/rusty-chess/commit/e837363ff2686f80c09aa72140b3d6afd39766a7))
* update opening book ([754f4fd](https://github.com/jwallace145/rusty-chess/commit/754f4fd9d6bb27335458413327c45938aa18d1a1))
* update pawn and rook position tables ([8513267](https://github.com/jwallace145/rusty-chess/commit/8513267ce610740c199a71932f05e155a0767146))
* use moves buffer during move generation for performance ([d4a7917](https://github.com/jwallace145/rusty-chess/commit/d4a791790682e21c754694cbaa5646939822c2e0))
