//! Atomic admission of runtime-validated restart checkpoints.
use crate::{Plan, Store, now};
use anyhow::Result;
use rusqlite::params;
impl Store {
    pub fn restore_checkpointed_tasks(&self, tasks: &[String], plans: &[Plan]) -> Result<usize> {
        let mut db = self.db()?;
        let tx = db.transaction()?;
        for plan in plans {
            tx.execute("UPDATE plans SET status=?2,body=?3,updated_at=?4 WHERE id=?1 AND status='interrupted'",params![plan.id,plan.status,serde_json::to_string(plan)?,now()])?;
        }
        let mut count = 0;
        for id in tasks {
            count+=tx.execute("UPDATE tasks SET status='queued',error=NULL,updated_at=?2 WHERE id=?1 AND status='interrupted'",params![id,now()])?;
            tx.execute("UPDATE subagents SET result=NULL WHERE id=?1 AND EXISTS(SELECT 1 FROM tasks WHERE tasks.id=?1 AND status='queued')",[id])?;
        }
        tx.commit()?;
        Ok(count)
    }
}
