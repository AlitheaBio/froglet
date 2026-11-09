# Synthetic immunopeptidomics fixtures

These files contain **invented data only**: 20 samples at two simulated sites, synthetic tissue labels and synthetic peptide detections. The candidate sequence is an example string with no asserted biological property. HLA names are standard allele labels; their use does not make the samples real.

- `fixture.json`: one query and four local SQLite tables per site. Each site contains exactly `samples`, `sample_hla_typing`, `sample_hla_typing_status` and `sample_peptide_detections`. Site labels are top-level metadata.
- `schema.sql`: the local table contract. It is not a production HLA-Compass database schema or data connector.
- `expected.json`: independently hand-labelled expected counts and sample membership lists for test review. Membership lists are synthetic development evidence and must not be included in provider results.

## What is counted

The query asks: among specimens in the exact sample-class/organ cohort whose recorded genotype includes the requested HLA allele, how many accepted available assay results contain the exact candidate sequence?

The unit is a **sample/specimen**, not a patient, donor, MS injection, technical replicate or peptide-spectrum match. These synthetic samples are distinct across sites. The example does not implement cross-site donor linkage or technical-replicate consolidation.

Input accepts an uppercase unmodified peptide sequence and an exact two-field class-I `HLA-A`, `HLA-B` or `HLA-C` name. It does not normalize aliases, truncate allele resolution, remove expression suffixes, match peptide variants or interpret PTM notation. `healthy_tissue` and `synthetic_organ_a` are exact synthetic metadata tokens, not an asserted universal vocabulary.

## HLA stratum and assay availability

HLA typing belongs to the sample. If the requested allele is recorded, that sample belongs to the stratum even when other typing information is incomplete. If it is not recorded, `complete` typing at the requested locus permits a nonmatching classification; `incomplete`, `missing` or absent completeness metadata produces an unknown classification. Here `missing` means no typing-completeness assessment is available. It does not override an actually recorded allele.

This is a **sample-genotype filter**. It does not establish that the selected allele presented the peptide or assign an observed peptide to one allele in a multiallelic sample.

`assay_status` is an explicit data-export assertion:

- `measured`: an accepted, completed assay's observation set is available in this local snapshot. An explicitly empty observation set is valid. Do not derive this status merely from the existence of one peptide row.
- `not_measured`: no accepted completed result is available because the assay was not performed or its result is unavailable.
- `failed`: the assay did not meet local acceptance criteria. Retained partial detection rows do not become accepted detections.

Eligible samples partition into measured, not measured and failed. The latter two contribute to missingness, not to the measured denominator. Among measured eligible samples, an exact candidate observation contributes one detected sample. Repeated observations never increase that sample's contribution. Absence from the accepted observation set means **not detected in that assay**, not biological absence.

## Hand-checked result

| Count | Site A | Site B | Sum |
|---|---:|---:|---:|
| Cohort samples | 8 | 8 | 16 |
| Unknown genotype stratum | 2 | 1 | 3 |
| Nonmatching genotype stratum | 1 | 1 | 2 |
| Eligible samples | 5 | 6 | 11 |
| Measured eligible samples | 3 | 5 | 8 |
| Detected | 2 | 3 | 5 |
| Not detected | 1 | 2 | 3 |
| Not measured | 1 | 1 | 2 |
| Failed | 1 | 0 | 1 |
| Missing | 2 | 1 | 3 |

The pooled detection fraction is **5/8 = 0.625**. Pool by summing counts; the unweighted mean of the site fractions is 19/30 and is deliberately different. Missingness is 3/11 among eligible samples. If the measured denominator is zero, the detection fraction is null, not zero. A missing/refusing site is not an empty cohort and cannot silently contribute zero to an all-sites result.

The fixture exercises duplicate detection rows, an accepted empty observation set, incomplete HLA typing both with and without the requested allele, missing typing, failed/unavailable assays, and samples outside the requested tissue/class cohort. A1's duplicate candidate observation counts once; A6/A7 detections do not enter the requested stratum's numerator.

## Limits

This workload counts previously accepted observations. It does not process spectra, choose false-discovery thresholds, validate local assay acceptance, harmonize intensity measurements or establish assay comparability across laboratories. Summing counts is arithmetic; it does not turn the fraction into population prevalence, immunogenicity, toxicity or therapeutic safety evidence.

Only aggregate service results are intended to leave each simulated site. This does not establish statistical privacy: small-cell suppression, adaptive-query limits and differential privacy are not supplied by the counting example. Real deployment requires its own data-access and result-release policy.

Primary research explaining why these distinctions matter:

- [Sensitive Immunopeptidomics by Leveraging Available Large-Scale Multi-HLA Spectral Libraries, Data-Independent Acquisition, and MS/MS Prediction](https://pmc.ncbi.nlm.nih.gov/articles/PMC8724634/) examines multiallelic samples and sensitivity dependent on measurement methods.
- [Deep learning boosts sensitivity of mass spectrometry-based immunopeptidomics](https://www.nature.com/articles/s41467-021-23713-9) shows that peptide identification changes with analytical sensitivity; non-detection is not a biological-absence guarantee.
