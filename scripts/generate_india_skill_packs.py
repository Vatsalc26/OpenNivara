from pathlib import Path
ROOT=Path('packs/builtin')
DENY='["write_file", "run_command", "open_url"]'
OFF={'JEE Main':['NTA JEE Main','JoSAA','CSAB'],'JEE Advanced':['JEE Advanced official website','JoSAA','AAT official notice'],'GATE':['GATE official website','Organising Institute','COAP','CCMT','PSU notices'],'NEET UG':['NTA NEET UG','MCC','NMC'],'NEET PG':['NBEMS','MCC','NMC'],'UPSC CSE':['UPSC','DoPT','Civil Services Examination Rules'],'CAT':['IIM CAT official website','IIM admission portals'],'CLAT':['Consortium of NLUs','NLU admission notices']}
REQ={'JEE Main':['jee','jee main','jee mains','nta'],'JEE Advanced':['jee advanced','advanced'],'GATE':['gate'],'NEET UG':['neet','neet ug'],'NEET PG':['neet pg'],'UPSC CSE':['upsc','civil services','cse'],'CAT':['cat','mba','iim'],'CLAT':['clat','nlu'],'General Study':[]}
PACKS={'india_student_essentials':('India Student Essentials','India Skills','Reusable data-only study skills for Indian students.'),'india_engineering_exams':('India Engineering Exams','India Exams','Data-only skill pack for JEE Main, JEE Advanced, and GATE preparation.'),'india_medical_exams':('India Medical Exams','India Exams','Data-only skill pack for NEET UG and NEET PG preparation.'),'india_upsc_cse':('India UPSC CSE','India Exams','Data-only skill pack for UPSC CSE preparation.'),'india_management_law_exams':('India Management and Law Exams','India Exams','Data-only skill pack for CAT and CLAT preparation.')}
ROWS='''india_student_essentials,india_study_plan_builder,General Study,planning
india_student_essentials,india_daily_timetable_builder,General Study,daily_schedule
india_student_essentials,india_revision_cycle_planner,General Study,revision
india_student_essentials,india_mock_test_analyzer,General Study,mock_analysis
india_student_essentials,india_weak_topic_diagnosis,General Study,diagnosis
india_student_essentials,india_notes_summarizer,General Study,notes
india_student_essentials,india_doubt_explainer,General Study,doubt_solving
india_student_essentials,india_exam_stress_consistency_coach,General Study,wellbeing
india_engineering_exams,jee_main_study_planner,JEE Main,planning
india_engineering_exams,jee_main_topic_prioritizer,JEE Main,topic_priority
india_engineering_exams,jee_main_mock_test_analyzer,JEE Main,mock_analysis
india_engineering_exams,jee_main_accuracy_speed_coach,JEE Main,accuracy_speed
india_engineering_exams,jee_main_formula_revision_system,JEE Main,formula_revision
india_engineering_exams,jee_main_application_admit_card_josaa_checklist,JEE Main,official_checklist
india_engineering_exams,jee_advanced_readiness_evaluator,JEE Advanced,readiness
india_engineering_exams,jee_advanced_deep_problem_solving_coach,JEE Advanced,problem_solving
india_engineering_exams,jee_advanced_pyq_pattern_analyzer,JEE Advanced,pyq_analysis
india_engineering_exams,jee_advanced_two_paper_strategy_coach,JEE Advanced,two_paper
india_engineering_exams,jee_advanced_error_notebook_builder,JEE Advanced,error_notebook
india_engineering_exams,jee_advanced_josaa_aat_next_step_checklist,JEE Advanced,official_checklist
india_engineering_exams,gate_paper_selection_goal_planner,GATE,paper_goal
india_engineering_exams,gate_branch_wise_study_planner,GATE,branch_planning
india_engineering_exams,gate_core_concepts_formula_coach,GATE,concept_formula
india_engineering_exams,gate_pyq_analyzer,GATE,pyq_analysis
india_engineering_exams,gate_mock_test_analyzer,GATE,mock_analysis
india_engineering_exams,gate_two_paper_combination_advisor,GATE,two_paper
india_engineering_exams,gate_mtech_psu_next_step_planner,GATE,official_next_steps
india_medical_exams,neet_ug_study_planner,NEET UG,planning
india_medical_exams,neet_ug_ncert_biology_mastery_coach,NEET UG,biology_ncert
india_medical_exams,neet_ug_physics_chemistry_numerical_coach,NEET UG,numericals
india_medical_exams,neet_ug_mock_test_analyzer,NEET UG,mock_analysis
india_medical_exams,neet_ug_revision_retention_planner,NEET UG,revision
india_medical_exams,neet_ug_application_admit_card_counselling_checklist,NEET UG,official_checklist
india_medical_exams,neet_pg_subject_backlog_triage,NEET PG,backlog
india_medical_exams,neet_pg_grand_test_analyzer,NEET PG,grand_test
india_medical_exams,neet_pg_clinical_vignette_reasoning_coach,NEET PG,clinical_reasoning
india_medical_exams,neet_pg_revision_cycle_planner,NEET PG,revision
india_medical_exams,neet_pg_internship_time_study_planner,NEET PG,internship_planning
india_medical_exams,neet_pg_counselling_branch_preference_checklist,NEET PG,official_checklist
india_upsc_cse,upsc_cse_foundation_planner,UPSC CSE,foundation
india_upsc_cse,upsc_prelims_strategy_coach,UPSC CSE,prelims
india_upsc_cse,upsc_csat_coach,UPSC CSE,csat
india_upsc_cse,upsc_mains_answer_writing_coach,UPSC CSE,mains_answer_writing
india_upsc_cse,upsc_essay_coach,UPSC CSE,essay
india_upsc_cse,upsc_optional_subject_planner,UPSC CSE,optional
india_upsc_cse,upsc_current_affairs_system_builder,UPSC CSE,current_affairs
india_upsc_cse,upsc_test_series_analyzer,UPSC CSE,test_series
india_upsc_cse,upsc_daf_interview_preparation_coach,UPSC CSE,interview
india_upsc_cse,upsc_form_filling_official_notice_checklist,UPSC CSE,official_checklist
india_management_law_exams,cat_study_planner,CAT,planning
india_management_law_exams,cat_varc_reading_coach,CAT,varc
india_management_law_exams,cat_dilr_set_selection_coach,CAT,dilr
india_management_law_exams,cat_qa_topic_prioritizer,CAT,qa_priority
india_management_law_exams,cat_mock_test_analyzer,CAT,mock_analysis
india_management_law_exams,cat_bschool_gdpi_wat_next_step_planner,CAT,official_next_steps
india_management_law_exams,clat_ug_study_planner,CLAT,planning_fresh
india_management_law_exams,clat_reading_comprehension_coach,CLAT,reading
india_management_law_exams,clat_legal_reasoning_coach,CLAT,legal_reasoning
india_management_law_exams,clat_logical_reasoning_coach,CLAT,logical_reasoning
india_management_law_exams,clat_gk_current_affairs_tracker,CLAT,gk_current_affairs
india_management_law_exams,clat_mock_test_analyzer,CLAT,mock_analysis
india_management_law_exams,clat_application_counselling_checklist,CLAT,official_checklist'''
SPECIAL={'jee_main_study_planner':['jee mains plan','90 day jee mains plan'],'jee_main_mock_test_analyzer':['jee main mock analysis','analyze my jee mock'],'jee_advanced_readiness_evaluator':['ready for jee advanced'],'jee_advanced_error_notebook_builder':['jee advanced error notebook'],'gate_pyq_analyzer':['gate cse pyq analysis'],'gate_two_paper_combination_advisor':['gate two paper combination da cse'],'neet_ug_ncert_biology_mastery_coach':['neet ug biology ncert revision plan'],'neet_ug_mock_test_analyzer':['neet mock score'],'neet_pg_grand_test_analyzer':['neet pg grand test analysis medicine weak'],'neet_pg_internship_time_study_planner':['internship neet pg plan'],'upsc_prelims_strategy_coach':['upsc prelims strategy'],'upsc_mains_answer_writing_coach':['upsc mains answer writing'],'cat_dilr_set_selection_coach':['cat dilr set selection strategy'],'cat_mock_test_analyzer':['cat mock varc low accuracy'],'clat_legal_reasoning_coach':['clat legal reasoning practice plan'],'clat_gk_current_affairs_tracker':['clat gk current affairs tracker'],'india_study_plan_builder':['study plan','30 day study plan'],'india_mock_test_analyzer':['analyze my mock']}
FRESH={'official_checklist','official_next_steps','paper_goal','two_paper','topic_priority','pyq_analysis','current_affairs','prelims','csat','optional','interview','qa_priority','planning_fresh'}
def q(s): return '"'+s.replace('"','\\"')+'"'
def arr(a): return '['+', '.join(q(x) for x in a)+']'
def name(id): return ' '.join({'jee':'JEE','neet':'NEET','ug':'UG','pg':'PG','upsc':'UPSC','cse':'CSE','gate':'GATE','cat':'CAT','clat':'CLAT','gk':'GK','qa':'QA','varc':'VARC','dilr':'DILR','pyq':'PYQ','ncert':'NCERT','josaa':'JoSAA','aat':'AAT','mtech':'MTech','psu':'PSU','gdpi':'GDPI','wat':'WAT','csat':'CSAT','daf':'DAF'}.get(p,p.capitalize()) for p in id.split('_'))
for pid,(pn,cat,desc) in PACKS.items():
    d=ROOT/pid; sd=d/'skills'; sd.mkdir(parents=True,exist_ok=True)
    (d/'pack.toml').write_text(f'''schema_version = 1\nid = {q(pid)}\nname = {q(pn)}\nversion = "1.0.0"\nauthor = "OpenNivara"\ncategory = {q(cat)}\ndescription = {q(desc)}\n[compatibility]\nopennivara_min_version = "0.1.0"\nopennivara_max_version = ""\n[contents]\nskills = true\npreferences = false\ncontexts = false\nstyle_presets = false\nprofile_templates = false\ntool_presets = false\nworkspace_map_rules = false\nprompt_behaviors = false\ncommand_snippets = false\ntheme = false\n[safety]\ncontains_executable_code = false\nmodifies_tool_permissions = false\nrequires_network = false\nrisk_level = "low"\n''')
    (d/'README.md').write_text('# '+pn+'\n\n'+desc+'\n\nData-only skill pack. Install in Store, enable in Settings -> Skills.\n')
for line in ROWS.splitlines():
    pid,id,exam,stage=line.split(','); nm=name(id); fresh=stage in FRESH; aliases=SPECIAL.get(id,[nm.lower()]); triggers=list(dict.fromkeys([exam.lower(),stage.replace('_',' '),*id.replace('_',' ').split(),*' '.join(aliases).split()]))
    neg=['react form','html form','login form','unit test'] if stage=='official_checklist' else []
    if id=='cat_varc_reading_coach': neg+=['mock','mock test']
    desc=f'Helps Indian students with {nm.lower()} through practical planning, analysis, and safe exam guidance.'
    prompt=f'Act as {nm} for Indian students. Use simple English, India-relevant terms like NCERT, PYQ, mock test, coaching, self-study, counselling, and official notices where relevant. Build practical today, 7 day, 30 day, and 90 day next steps. Do not invent official facts or promise outcomes.'
    src=OFF.get(exam,[]) if fresh else []
    (ROOT/pid/'skills'/f'{id}.toml').write_text(f'''schema_version = 1\nid = {q(id)}\npack_id = {q(pid)}\nname = {q(nm)}\ndescription = {q(desc)}\nenabled = false\ncategory = {q('india_student' if pid=='india_student_essentials' else 'india_exams')}\nroute_policy = "auto"\naliases = {arr(aliases)}\ntriggers = {arr(triggers)}\nrequired_any = {arr(REQ[exam])}\nnegative_triggers = {arr(neg)}\nexamples = {arr(aliases)}\nmin_score = 25\n[prompt]\nrole = {q(nm+' for Indian students')}\ninstructions = {q(prompt)}\nconstraints = ["Do not invent official dates, fees, cutoffs, seats, eligibility, deadlines, answer keys, or counselling rules.", "Do not promise rank, percentile, admission, selection, or score jumps.", "Use Hinglish only if the user asks or uses Hinglish."]\n[tools]\nallow = []\ndeny = {DENY}\n[safety]\nrisk_level = "low"\nrequires_confirmation = false\nallows_file_write = false\nallows_shell = false\nallows_network = false\nrequires_fresh_info = {'true' if fresh else 'false'}\n[metadata]\naudience = ["student", "exam_aspirant"]\ncountry = "IN"\nexam = {q(exam)}\nexam_stage = {q(stage)}\nlanguage_style = ["english", "hinglish_optional"]\nlast_reviewed_at = "2026-06-03"\nfreshness_sensitive = {'true' if fresh else 'false'}\nofficial_source_labels = {arr(src)}\n[store_preview]\nbest_for = {arr([desc])}\nnot_for = ["Official notice replacement", "Guaranteed rank or admission"]\nsample_prompts = {arr(aliases)}\nwhat_it_will_do = ["Builds practical next steps", "Flags official verification needs"]\nwhat_it_will_not_do = ["Invent official facts", "Promise outcomes"]\n''')
print('generated 62')
