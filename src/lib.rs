use anchor_lang::prelude::*;

declare_id!("ECMsfGwnQZ7NhccZzyWd1WBKXNmLadeC2nLKCh8wXkNQ");

#[program]
pub mod auditrace {
    use super::*;

    /* Define la funcion que crea un registro principal de auditoria */

    pub fn crear_registry(ctx: Context<CrearRegistry>, name: String) -> Result<()> {

        // Permite obtener y modificar la cuenta donde se guardan los datos del registro
        let registry = &mut ctx.accounts.registry;

        registry.owner = ctx.accounts.owner.key(); // Guarda el creador del registro como propietario
        registry.name = name; // Nombre del identificador de la bd auditada
        registry.records = Vec::new(); // Vector que almacenara los registros referentes al objeto auditado
        registry.counter = 0; // Contador que ayuda a generar PDAs unicas para los registros

        msg!("Registry creado!");

        Ok(())
    }

    /* Crea un subregistro auditado */

    pub fn crear_record(
        ctx: Context<CrearRecord>,
        title: String, // Nombre del registro
        hash: [u8;32], // datos auditados
    ) -> Result<()> {

        let registry = &mut ctx.accounts.registry; // Obtener la cuenta principal del registro

        // verificar que solo el propietario pueda modificar el registro
        require!(
            registry.owner == ctx.accounts.owner.key(),
            ErrorCode::NotOwner
        );

        // Obtener y modificar la nueva cuenta del subregistro
        let record = &mut ctx.accounts.record;

        record.registry = registry.name.clone(); // Guarda el nombre del registro al que petenece
        record.title = title.clone(); // Guardar el nombre del registro creado
        record.hash = hash; // Guardar el hash de los datos auditaos
        record.active = true; // Marcar el registro como activo/existente
        record.counter = 0; // Contador que permite generar PDAs unicar para los eventos

        registry.records.push(record.key()); // Guardar la direccion del subregistro en el vector del registro principal
        registry.counter += 1; // Incrementar el contador de registros

        let clock = Clock::get()?; // Obtiene el timestamp del bloque actual

        // Obtener la cuenta del evento
        let event = &mut ctx.accounts.event;

        event.record = record.key(); // Guardar la direccion del subregistro
        event.action = "CREATE".to_string(); // Definir el tipo del evento (En este caso estamos creando un subregistro)
        event.actor = ctx.accounts.owner.key(); // Guardar al responsable de la accion
        event.timestamp = clock.unix_timestamp; // Guardar el momento en que ocurrio el evento
        record.counter += 1; // Incrementar contador despues de crear el evento

        msg!("Record creado!");

        Ok(())
    }

    /* Permite modificar el hash del subregistro */

    pub fn actualizar_record(
        ctx: Context<ActualizarRecord>,
        new_hash: [u8;32],
    ) -> Result<()> {

        // Obtener la cuenta del subregistro
        let record = &mut ctx.accounts.record;

        record.hash = new_hash; // Actualizar el hash

        let clock = Clock::get()?; // Obtener el timestamp

        // Crear un nuevo evento para la actualizacion
        let event = &mut ctx.accounts.event;

        event.record = record.key(); // Guardar la direccion del subregistro afectado
        event.action = "UPDATE".to_string(); // Indicar que el evento actualizo al subregistro
        event.actor = ctx.accounts.owner.key(); // Guardar al responsable
        event.timestamp = clock.unix_timestamp; // Guardar el momento
        record.counter += 1; // Actualizar el contador

        msg!("Record actualizado");

        Ok(())
    }

    /* Eliminar (desactivar) un subregistro*/

    pub fn eliminar_record(
        ctx: Context<EliminarRecord>,
    ) -> Result<()> {
        
        // Obtener la cuenta del subregistro
        let record = &mut ctx.accounts.record;
        
        record.active = false; // Desactivar/eliminar el subregistro

        let clock = Clock::get()?; // Obtener el momento

        // Crear evento de eliminacion
        let event = &mut ctx.accounts.event;

        event.record = record.key(); // Guardar la direccion del subregistro
        event.action = "DELETE".to_string(); // Indicar que el evento "elimino" al subregistro
        event.actor = ctx.accounts.owner.key(); // responsable
        event.timestamp = clock.unix_timestamp; // tiempo
        record.counter += 1; // Actualizar el contador

        msg!("Record eliminado");

        Ok(())
    }
}

/* Imprimir error en caso de que alguien externo intente modificar
    un registro que no le pertenece
 */

#[error_code] // Permite definir errores personalisados
pub enum ErrorCode {

    #[msg("No eres el owner >:(")]
    NotOwner,
}

/* Definir las estructuras de los datos a almacenar */

// Cuenta principal del sistema
#[account]
#[derive(InitSpace)]
pub struct AuditRegistry {

    pub owner: Pubkey, // Wallet del propietario

    #[max_len(60)]
    pub name: String, // Nopmbre del objeto a auditar

    #[max_len(100)] // Ejemplo basico para no usar mucho espacio :c
    pub records: Vec<Pubkey>, // Vector para las direcciones de los subregistros

    pub counter: u64, // Contador para generar PDAs unicas
}

// Subregitro auditado
#[account]
#[derive(InitSpace)]
pub struct AuditRecord {

    #[max_len(60)]
    pub registry: String, // Registro padre ( ͡° ͜ʖ ͡°)

    #[max_len(60)]
    pub title: String, // Nombre del subregistro

    pub hash: [u8;32], // hash de los datos

    pub active: bool, // Indica si el registro existe

    pub counter: u64, // Contador para generar PDAs unicas en el registro de eventos
}

// Accion/evento que ocurre sobre el registro
#[account]
#[derive(InitSpace)]
pub struct AuditEvent {

    pub record: Pubkey, // Identificador del registro afectado

    #[max_len(10)]
    pub action: String, // Tipo de evento (Create, Update, Delete)

    pub actor: Pubkey, // Responsable del evento

    pub timestamp: i64, // Fecha del evento
}

/* Definir los contextos (las cuentas que necesita cada instruccion) */

#[derive(Accounts)]
#[instruction(name:String)]
pub struct CrearRegistry<'info> { // Cuentas necesarias para crear un registro

    #[account(mut)]
    pub owner: Signer<'info>, // la wallet que firma la transaccion (dueño)

    #[account(
        init, // Indica que se creara la cuenta
        payer = owner, // El dueño la paga
        space = 8 + AuditRegistry::INIT_SPACE,
        seeds = [b"registry", name.as_bytes(), owner.key().as_ref()], // Definir las semillas para generar la PDA
        bump
    )]
    pub registry: Account<'info, AuditRegistry>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(title:String)]
pub struct CrearRecord<'info> { // Cuentas necesarias para crear un Subregistro

    #[account(mut)]
    pub owner: Signer<'info>, // La wallet que firma la transaccion (dueño)

    #[account(mut)]
    pub registry: Account<'info, AuditRegistry>, // El registro padre

    #[account(
        init, // Indica que se creara la cuenta
        payer = owner, // El dueño paga
        space = 8 + AuditRecord::INIT_SPACE,
        seeds = [ // Definir las semillas para generar la PDA
            b"record",
            owner.key().as_ref(),
            registry.counter.to_le_bytes().as_ref() // Utilizar el contador del registro para generar la PDA
        ],
        bump
    )]
    pub record: Account<'info, AuditRecord>,
    // Definir lo necesario para crear un evento
    #[account(
        init, // Inicializar el evento
        payer = owner, // El dueño paga
        space = 8 + AuditEvent::INIT_SPACE,
        seeds = [
            b"event",
            record.key().as_ref(),
            record.counter.to_le_bytes().as_ref(),
            b"create"
        ],
        bump
    )]
    pub event: Account<'info, AuditEvent>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ActualizarRecord<'info> { // Cuentas necesarias para actualizar un Subregistro

    #[account(mut)]
    pub owner: Signer<'info>, // La wallet

    #[account(mut)]
    pub record: Account<'info, AuditRecord>, // El papa registro

    #[account(
        init, // Inicializar
        payer = owner, // Dueño paga
        space = 8 + AuditEvent::INIT_SPACE,
        seeds = [
            b"event",
            record.key().as_ref(),
            record.counter.to_le_bytes().as_ref(),
            b"update"
        ],
        bump
    )]
    pub event: Account<'info, AuditEvent>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct EliminarRecord<'info> { // Cuentas necesarias para eliminar un Subregistro

    #[account(mut)]
    pub owner: Signer<'info>, // Wallet

    #[account(mut)]
    pub record: Account<'info, AuditRecord>, // Papa

    #[account(
        init, // Inicio
        payer = owner, // Dueño Paga
        space = 8 + AuditEvent::INIT_SPACE,
        seeds = [
            b"event",
            record.key().as_ref(),
            record.counter.to_le_bytes().as_ref(),
            b"delete"
        ],
        bump
    )]
    pub event: Account<'info, AuditEvent>,

    pub system_program: Program<'info, System>,
}