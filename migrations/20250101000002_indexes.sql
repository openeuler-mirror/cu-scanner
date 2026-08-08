-- Indexes for cu-scanner
-- All indexes matching design doc Section 5.3

CREATE INDEX IF NOT EXISTS idx_csaf_sources_oval_numeric_id ON csaf_sources(oval_numeric_id);
CREATE INDEX IF NOT EXISTS idx_csaf_sources_release_date ON csaf_sources(release_date);

CREATE INDEX IF NOT EXISTS idx_oval_definitions_oval_id ON oval_definitions(oval_id);
CREATE INDEX IF NOT EXISTS idx_oval_definitions_csaf_id ON oval_definitions(csaf_id);
CREATE INDEX IF NOT EXISTS idx_oval_definitions_issued_date ON oval_definitions(issued_date);

CREATE INDEX IF NOT EXISTS idx_oval_references_definition_id ON oval_references(definition_id);
CREATE INDEX IF NOT EXISTS idx_oval_cves_definition_id ON oval_cves(definition_id);
CREATE INDEX IF NOT EXISTS idx_oval_cves_cve_id ON oval_cves(cve_id);
CREATE INDEX IF NOT EXISTS idx_oval_cpes_definition_id ON oval_cpes(definition_id);

CREATE INDEX IF NOT EXISTS idx_oval_tests_definition_id ON oval_tests(definition_id);

CREATE INDEX IF NOT EXISTS idx_oval_criteria_definition_id ON oval_criteria(definition_id);
CREATE INDEX IF NOT EXISTS idx_oval_criteria_parent_id ON oval_criteria(parent_id);

CREATE INDEX IF NOT EXISTS idx_download_tasks_file_name ON download_tasks(file_name);
CREATE INDEX IF NOT EXISTS idx_download_tasks_sync_batch_id ON download_tasks(sync_batch_id);
CREATE INDEX IF NOT EXISTS idx_download_tasks_status ON download_tasks(status);
CREATE INDEX IF NOT EXISTS idx_download_tasks_created_at ON download_tasks(created_at);
