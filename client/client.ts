import { PublicKey } from "@solana/web3.js";

const owner = pg.wallet.publicKey; // Obtener la wallet del usuario

const registryName = "AudiTrace"; // Nombre de mi sistema de Auditoria

console.log("Wallet:", owner.toString());

// Generar pda del Registro Papa
function pdaRegistry(name) {
  return PublicKey.findProgramAddressSync(
    [
      Buffer.from("registry"), // Primera semilla
      Buffer.from(name), // Segunda semilla
      owner.toBuffer() // Tercer semilla <- se asegura de que cada usuario tenga sus propios registros
    ],
    pg.PROGRAM_ID
  );
}

// Generar cuenta para cada subregistro (auditorias)
function pdaRecord(counter) {
  return PublicKey.findProgramAddressSync(
    [
      Buffer.from("record"),
      owner.toBuffer(),
      new anchor.BN(counter).toArrayLike(Buffer, "le", 8) // convertir el contador a bytes <- asegurarse de evitar colisiones
    ],
    pg.PROGRAM_ID
  );
}

// Generar cuenta para cada evento
function pdaEvent(record, counter, action) {

  return PublicKey.findProgramAddressSync(
    [
      Buffer.from("event"),
      record.toBuffer(),
      new anchor.BN(counter).toArrayLike(Buffer, "le", 8),
      Buffer.from(action)
    ],
    pg.PROGRAM_ID
  );
}

// Crear registry
async function crearRegistry(registryName) {

  const [registry] = pdaRegistry(registryName);

  const tx = await pg.program.methods // Llama a la isntruccion del smartcontract
    .crearRegistry(registryName)
    .accounts({ // Define las cuentas por transaccion
      owner,
      registry
    })
    .rpc(); // Enviar a la blockchain

  console.log("TX:", tx);
}

// Crear subregistro
async function crearRecord(title, hash, registryName) {

  const [registry] = pdaRegistry(registryName);

  const registryAccount = await pg.program.account.auditRegistry.fetch(registry); // Obtener el papa

  const counter = registryAccount.counter;// Obtener el contador

  // Generar los PDAs
  const [record] = pdaRecord(counter); 

  const [event] = pdaEvent(record, 0, "create");

  const tx = await pg.program.methods
    .crearRecord(title, hash)
    .accounts({ // Definir las cuentas
      owner,
      registry,
      record,
      event
    })
    .rpc();

  console.log("Record creado:", tx);
}

// Actualizar subregistro
async function actualizarRecord(record, hash) {

  const recordAccount = await pg.program.account.auditRecord.fetch(record); // Obtener el record

  const record_counter = recordAccount.counter; // Obtener el contador del record

  const [event] = pdaEvent(record, record_counter,"update"); // Crear PDA del evento

  const tx = await pg.program.methods
    .actualizarRecord(hash)
    .accounts({ // Cuentas
      owner,
      record,
      event
    })
    .rpc();

  console.log("Record actualizado:", tx);
}

// Eliminar subregistro
async function eliminarRecord(record) {

  const recordAccount = await pg.program.account.auditRecord.fetch(record); // Obtener el record

  const record_counter = recordAccount.counter; // Obtener el contador del record

  const [event] = pdaEvent(record, record_counter,"delete"); // PDA del evento

  const tx = await pg.program.methods
    .eliminarRecord()
    .accounts({ // Cuentas
      owner,
      record,
      event
    })
    .rpc();

  console.log("Record eliminado:", tx);
}

// Listar los subregistros existentes segun el registro padre
async function verRecords(registryName) {

  const [registry] = pdaRegistry(registryName);

  try {

    const registryAccount = await pg.program.account.auditRegistry.fetch(registry); // Obtener subregistros segun el registro padre

    const numeroRecords = registryAccount.records.length; // Cantidad de registros

    if (!registryAccount.records || numeroRecords === 0) { // No se encontraron registros
      console.log("No hay registros auditados");
      return;
    }

    for (let i = 0; i < numeroRecords; i++) {

      const recordKey = registryAccount.records[i];

      const recordAccount = await pg.program.account.auditRecord.fetch(recordKey);

      // Imprimir informacion de los subregistros
      console.log(
        `Record #${i + 1}:
        * Título: ${recordAccount.title}
        * Registry: ${recordAccount.registry}
        * Hash: ${Buffer.from(recordAccount.hash).toString("hex")}
        * Activo: ${recordAccount.active}
        * PDA: ${recordKey.toBase58()}`
      );

    }

  } catch (error) { // Imprimir errores
    console.error("Error viendo records:", error);
    if (error.message) {
      console.error("Mensaje:", error.message);
    }
    if (error.logs) {
      console.error("Logs:", error.logs);
    }
  }
}

// Listar los eventos que afectaron a un subregistro
async function verEventos(record) {

  const recordAccount = await pg.program.account.auditRecord.fetch(record);

  const total = recordAccount.counter; // Obtener la cantidad de eventos

  const actions = ["create", "update", "delete"]; // Las 3 acciones posibles de los eventos

  for (let i = 0; i < total.toNumber(); i++) {

    for (const action of actions) {

      try { // Intentar las 3 acciones sin romper el programa

        const [event] = pdaEvent(record, i, action);

        const eventAccount = await pg.program.account.auditEvent.fetch(event);
        
        // Imprimir informacion de los eventos
        console.log(
          `Evento #${i + 1}
           Acción: ${eventAccount.action}
           Actor: ${eventAccount.actor.toBase58()}
           Timestamp: ${eventAccount.timestamp}`
        );

        break;

      } catch (err) {
        // Si no existe el PDA se ignora
      }

    }

  }

}

// Ejemplo de ejecucion para las pruebas, PD. Jalo a la primera 😁

// (async () => {
//     // Crear un registro principal (BD a auditar)
//     await crearRegistry("Diffuzor");

//     // Crear un subregistro (Query que afecto a la bd)
//     const hash = new Uint8Array([225,96,151,248,80,144,131,158,71,146,22,240,96,166,98,29,178,226,8,206,1,9,10,187,17,112,196,32,4,80,95,232]); // Ejemplo de hash
//     await crearRecord("TABLE Productos", hash, "Diffuzor");

//     // Ver todos los subregistros
//     await verRecords("Diffuzor");

//     // Obtener la PDA del primer subregistro para actualizarlo/eliminarlo
//     const [registry] = pdaRegistry("Diffuzor");
//     const registryAccount = await pg.program.account.auditRegistry.fetch(registry);
//     const firstRecord = registryAccount.records[0];

//     // Actualizar el primer subregistro
//     const newHash = new Uint8Array([140, 39, 143, 178, 128, 196, 190, 19, 28, 100, 174, 96, 107, 54, 177, 87, 20, 63, 156, 67, 203, 216, 58, 180, 198, 176, 55, 84, 184, 187, 222, 42]);
//     await actualizarRecord(firstRecord, newHash);

//     // Ver eventos del primer subregistro
//     await verEventos(firstRecord);

//     // Eliminar el primer subregistro
//     await eliminarRecord(firstRecord);

//     // Ver nuevamente los subregistros
//     await verRecords("Diffuzor");
// })();