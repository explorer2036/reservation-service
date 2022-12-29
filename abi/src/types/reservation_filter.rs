use crate::{
    Normalizer, ReservationFilter, ReservationFilterBuilder, ReservationStatus, ToSql, Validator,
};

impl ReservationFilterBuilder {
    pub fn build(&self) -> Result<ReservationFilter, crate::Error> {
        let mut filter = self
            .private_build()
            .expect("failed to build ReservationFilter");
        filter.normalize()?;
        Ok(filter)
    }
}

impl Validator for ReservationFilter {
    fn validate(&self) -> Result<(), crate::Error> {
        if self.page_size < 10 || self.page_size > 100 {
            return Err(crate::Error::InvalidPageSize(self.page_size));
        }
        if let Some(cursor) = self.cursor {
            if cursor < 0 {
                return Err(crate::Error::InvalidCursor(cursor));
            }
        }
        ReservationStatus::from_i32(self.status).ok_or(crate::Error::InvalidStatus(self.status))?;
        Ok(())
    }
}

impl Normalizer for ReservationFilter {
    fn do_normalize(&mut self) {
        if self.status == ReservationStatus::Unknown as i32 {
            self.status = ReservationStatus::Pending as i32;
        }
    }
}

impl ReservationFilter {
    pub fn get_cursor(&self) -> i64 {
        self.cursor.unwrap_or(if self.desc { i64::MAX } else { 0 })
    }

    pub fn get_status(&self) -> ReservationStatus {
        ReservationStatus::from_i32(self.status).unwrap()
    }
}

impl ToSql for ReservationFilter {
    fn to_sql(&self) -> String {
        let limit = self.page_size;
        let status = self.get_status();

        let cursor_cond = if self.desc {
            format!("id <= {}", self.get_cursor())
        } else {
            format!("id >= {}", self.get_cursor())
        };

        let user_resource_cond = match (self.user_id.is_empty(), self.resource_id.is_empty()) {
            (true, true) => "TRUE".to_string(),
            (true, false) => format!("resource_id = '{}'", self.resource_id),
            (false, true) => format!("user_id = '{}'", self.user_id),
            (false, false) => format!(
                "user_id = '{}' AND resource_id = '{}'",
                self.user_id, self.resource_id
            ),
        };

        let direction = if self.desc { "DESC" } else { "ASC" };

        format!("SELECT * FROM reservations WHERE status = '{}'::reservation_status AND {} AND {} ORDER BY id {} LIMIT {}", status, cursor_cond, user_resource_cond, direction, limit)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ReservationFilterBuilder;

    #[test]
    fn filter_should_generate_correct_sql() {
        let filter = ReservationFilterBuilder::default()
            .user_id("alon")
            .build()
            .unwrap();
        let sql = filter.to_sql();
        assert_eq!(sql, "SELECT * FROM reservations WHERE status = 'pending'::reservation_status AND id >= 0 AND user_id = 'alon' ORDER BY id ASC LIMIT 10");

        let filter = ReservationFilterBuilder::default()
            .user_id("alon")
            .resource_id("test")
            .build()
            .unwrap();
        let sql = filter.to_sql();
        assert_eq!(sql, "SELECT * FROM reservations WHERE status = 'pending'::reservation_status AND id >= 0 AND user_id = 'alon' AND resource_id = 'test' ORDER BY id ASC LIMIT 10");

        let filter = ReservationFilterBuilder::default()
            .desc(true)
            .build()
            .unwrap();
        let sql = filter.to_sql();
        assert_eq!(sql, "SELECT * FROM reservations WHERE status = 'pending'::reservation_status AND id <= 9223372036854775807 AND TRUE ORDER BY id DESC LIMIT 10");

        let filter = ReservationFilterBuilder::default()
            .user_id("alon")
            .cursor(10)
            .desc(true)
            .build()
            .unwrap();
        let sql = filter.to_sql();
        assert_eq!(sql, "SELECT * FROM reservations WHERE status = 'pending'::reservation_status AND id <= 10 AND user_id = 'alon' ORDER BY id DESC LIMIT 10");
    }
}
