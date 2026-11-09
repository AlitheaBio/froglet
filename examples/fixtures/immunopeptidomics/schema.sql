PRAGMA foreign_keys = ON;

CREATE TABLE samples (
  sample_id TEXT PRIMARY KEY,
  sample_class TEXT NOT NULL,
  organ TEXT NOT NULL,
  assay_status TEXT NOT NULL CHECK (assay_status IN ('measured', 'not_measured', 'failed'))
);
CREATE TABLE sample_hla_typing (
  sample_id TEXT NOT NULL REFERENCES samples(sample_id),
  allele_name TEXT NOT NULL
);
CREATE TABLE sample_hla_typing_status (
  sample_id TEXT NOT NULL REFERENCES samples(sample_id),
  locus TEXT NOT NULL,
  status TEXT NOT NULL CHECK (status IN ('complete', 'incomplete', 'missing')),
  PRIMARY KEY (sample_id, locus)
);
CREATE TABLE sample_peptide_detections (
  sample_id TEXT NOT NULL REFERENCES samples(sample_id),
  sequence TEXT NOT NULL
);
