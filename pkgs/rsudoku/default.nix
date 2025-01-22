{ lib
, rustPlatform
}:

rustPlatform.buildRustPackage {
  pname = "rsudoku";
  version = "0.1.0";
  src = ../../.;
  cargoLock.lockFile = ../../Cargo.lock;

  meta = {
    description = "Full-featured sudoku game and solver implemented in Rust";
    homepage = "";
    license = lib.licenses.gpl3Plus;
    maintainers = with lib.maintainers; [ oosquare ];
    mainProgram = "rsudoku";
    platforms = lib.platforms.all;
  };
}
